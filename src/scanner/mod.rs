//! src/scanner/mod.rs
//!
//! This module coordinates the whole scanning process:
//!   - Load the wordlist from disk
//!   - Parse extra extensions from CLI flags
//!   - Build absolute target URLs to probe
//!   - Run HTTP probes with bounded concurrency (semaphore)
//!   - Print only “interesting” responses (200/301/302/401/403)
//!
//! The heavy I/O work is delegated to submodules:
//!   - wordlist.rs : reading and filtering wordlist lines
//!   - targets.rs  : turning (base + words + exts) into absolute URLs
//!   - http.rs     : performing one HTTP probe and summarizing the response
//!   - util.rs     : small helpers (timestamp, status filtering)

use crate::{args::Args, error::DirustError};
use reqwest::Client;
use std::sync::Arc;
use tokio::{sync::Semaphore, task::JoinHandle};

// Bring in submodules that this orchestrator relies on.
mod wordlist;
mod targets;
mod http;
mod util;

// Types and helpers used locally from the submodules.
use http::HttpSummary;
use util::{is_interesting_status, timestamp_seconds};

/// Run the full scan using a pre-built HTTP client, a normalized base URL,
/// and the parsed CLI arguments.
///
/// Returns:
///   - Ok(()) on success (including the case where zero targets were “interesting”)
///   - Err(DirustError) if any fatal error occurs (file I/O, HTTP, or task join failure)
pub async fn scan(client: &Client, base: &str, args: &Args) -> Result<(), DirustError> {
    // 1) Read wordlist from disk and apply basic filtering (trim, skip empty/#comment).
    //    Any I/O error (e.g., file not found, permission denied) is returned immediately.
    let words = wordlist::read_wordlist(&args.wordlist)?;

    // 2) Parse the comma-separated extensions passed via CLI into a normalized Vec<String>.
    //    Example: "php,html,txt" -> [".php", ".html", ".txt"]
    let extensions = args.parse_exts();

    // 3) Build the final list of absolute URLs to probe (base + word [+ ext]).
    //    The target builder ensures we do not add extensions to directories (“admin/”)
    //    or to words that already contain a dot (“readme.txt”).
    let all_targets = targets::build_targets(base, &words, &extensions);

    // 4) Prepare bounded concurrency using a semaphore.
    //    We acquire a permit BEFORE spawning each task, guaranteeing that the number of
    //    in-flight requests never exceeds `args.concurrency`.
    let semaphore = Arc::new(Semaphore::new(args.concurrency));

    // We store the JoinHandle of each spawned task so we can await them and propagate errors.
    let mut jobs: Vec<JoinHandle<Result<(), DirustError>>> = Vec::with_capacity(all_targets.len());

    // Iterate the full list of targets and schedule each probe as an async task.
    for url in all_targets {
        // Try to acquire a concurrency permit. If this fails (which is rare and indicates
        // the semaphore was closed), we log and skip scheduling this target.
        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => {
                eprintln!("[!] failed to acquire semaphore permit");
                continue;
            }
        };

        // Clone the shared client for this task. `reqwest::Client` is cheap to clone:
        // it shares connection pools and other internals under the hood.
        let client_clone = client.clone();

        // Record whether we should use GET instead of HEAD, as requested by the CLI.
        let use_get = args.get;

        // Spawn one asynchronous task per target.
        // The `_permit` binding is kept inside the task so the permit is released when
        // the task completes (drop semantics).
        let handle: JoinHandle<Result<(), DirustError>> = tokio::spawn(async move {
            // Keep the permit alive for the lifetime of this task.
            let _permit = permit;

            // Perform a single HTTP probe for the given URL.
            // - Uses HEAD by default (fast, no body)
            // - Falls back to GET on 405 (Method Not Allowed), or always uses GET if requested
            let probe_result = http::probe(&client_clone, &url, use_get).await?;

            // Decide whether to print this line based on the status code.
            // We only print “interesting” statuses: 200, 301, 302, 401, 403.
            if is_interesting_status(probe_result.status) {
                print_line(&url, &probe_result);
            }

            // Task completed successfully.
            Ok(())
        });

        // Keep the task handle to await it later.
        jobs.push(handle);
    }

    // 5) Await all spawned tasks and propagate the first error we encounter.
    //    This ensures that if a task returns an error (e.g., HTTP client error),
    //    we abort the scan with a clear message rather than silently ignoring it.
    for handle in jobs {
        // `handle.await` can fail if the task panicked or was cancelled.
        match handle.await {
            // The task ran to completion. Now inspect the inner Result.
            Ok(inner_result) => {
                // We avoid the `if let` shortcut and use a full `match` for clarity.
                match inner_result {
                    Ok(()) => {
                        // Task returned Ok — nothing to do.
                    }
                    Err(e) => {
                        // Task returned an application error (e.g., HTTP or I/O).
                        // Bubble it up so `main` can report it and exit non-zero.
                        return Err(e);
                    }
                }
            }
            // The task did not run to a normal completion (panic or cancellation).
            Err(join_err) => {
                return Err(DirustError::from(join_err));
            }
        }
    }

    // If we get here, all tasks finished and none reported an error.
    Ok(())
}

/// Print one result line in a consistent, grep-friendly format.
///
/// Format:
///   [<unix_ts>] <status> len=<Content-Length or "-">  <url> [-> <Location>]
///
/// Examples:
///   [1712345678] 200 len=1234  https://example.com/admin
///   [1712345679] 301 len=-     https://example.com/admin -> https://example.com/admin/
fn print_line(url: &str, summary: &HttpSummary) {
    // Prepare values for printing:
    // - UNIX timestamp (seconds) for easy chronological sorting
    // - status code as a u16 (e.g., 200, 301)
    // - content-length as a string, or "-" if unknown
    let ts = timestamp_seconds();
    let status = summary.status.as_u16();
    let len_str = match &summary.content_length {
        Some(s) => s.as_str(),
        None => "-",
    };

    // Print with or without the redirect target depending on whether Location is present.
    match &summary.location {
        Some(loc) => {
            println!(
                "[{}] {:>3} len={}  {} -> {}",
                ts, status, len_str, url, loc
            );
        }
        None => {
            println!(
                "[{}] {:>3} len={}  {}",
                ts, status, len_str, url
            );
        }
    }
}
