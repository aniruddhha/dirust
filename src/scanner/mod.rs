use crate::{args::Args, error::DirustError};
use reqwest::Client;
use std::sync::Arc;
use tokio::{sync::Semaphore, task::JoinHandle};

mod wordlist;
mod targets;
mod http;
mod util;

use http::HttpSummary;
use util::{is_interesting_status, timestamp_seconds};

pub async fn scan(client: &Client, base: &str, args: &Args) -> Result<(), DirustError> {
    // 1) Read wordlist
    let words = wordlist::read_wordlist(&args.wordlist)?;

    // 2) Parse extensions from CLI
    let extensions = args.parse_exts();

    // 3) Build all target URLs to probe
    let all_targets = targets::build_targets(base, &words, &extensions);

    // 4) Run probes with bounded concurrency
    let semaphore = Arc::new(Semaphore::new(args.concurrency));
    let mut jobs: Vec<JoinHandle<Result<(), DirustError>>> = Vec::with_capacity(all_targets.len());

    for url in all_targets {
        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => {
                eprintln!("[!] failed to acquire semaphore permit");
                continue;
            }
        };

        let client_clone = client.clone();
        let use_get = args.get;

        let handle: JoinHandle<Result<(), DirustError>> = tokio::spawn(async move {
            let _permit = permit;
            let probe = http::probe(&client_clone, &url, use_get).await?;
            if is_interesting_status(probe.status) {
                print_line(&url, &probe);
            }
            Ok(())
        });

        jobs.push(handle);
    }

    // 5) Await all tasks and propagate the first error, if any
    for handle in jobs {
        match handle.await {
            Ok(inner_result) => {
                if let Err(e) = inner_result {
                    return Err(e);
                }
            }
            Err(join_err) => {
                // Task panicked or was cancelled
                return Err(DirustError::from(join_err));
            }
        }
    }

    Ok(())
}

fn print_line(url: &str, summary: &HttpSummary) {
    // Prepare printable pieces
    let ts = timestamp_seconds();
    let status = summary.status.as_u16();
    let len_str = match &summary.content_length {
        Some(s) => s.as_str(),
        None => "-",
    };

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