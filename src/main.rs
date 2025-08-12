//! src/main.rs
//!
//! Entry point for the Dirust binary.
//!
//! Responsibilities of this file:
//!   1) Declare the modules used by the program (`args`, `error`, `scanner`, `url`).
//!   2) Parse command-line arguments into a typed `Args` struct (via `clap`).
//!   3) Normalize and validate the base URL (HTTP/HTTPS + trailing slash).
//!   4) Build a reusable HTTP client (`reqwest::Client`) with sane defaults.
//!   5) Start the asynchronous scan and return any error to the OS.
//!
//! Notes:
//!   - We use Tokio's multi-thread runtime to drive async I/O across several worker threads.
//!   - `main` returns `Result<(), DirustError>` so we can bubble up failures cleanly.

mod args;     // CLI definition and helpers (parse flags/positional args)
mod error;    // Central application error type (`DirustError`)
mod scanner;  // Orchestrates wordlist read, target build, concurrency, probing, and printing
mod url;      // Base URL validation and normalization

use args::Args;                 // Parsed CLI arguments (from `src/args.rs`)
use clap::Parser;               // `Args::parse()` derive support from clap
use error::DirustError;         // Our explicit error type for clean propagation
use reqwest::Client;            // HTTP client (connection pooling, TLS, etc.)

/// The Tokio runtime macro sets up an async executor for us.
/// `flavor = "multi_thread"` starts a pool of worker threads (typically = CPU cores),
/// which is ideal for high-concurrency network I/O.
///
/// Returning `Result<(), DirustError>` allows us to use `?` inside `main` and have
/// any error automatically turned into a non-zero process exit.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), DirustError> {
    // Parse command-line flags and positional arguments into a strongly-typed struct.
    // Example CLI:
    //   dirust https://example.com/ -w words.txt --exts php,html -c 100 --get
    let args: Args = Args::parse();

    // Validate the base URL and ensure it ends with a trailing slash `/`.
    // This prevents mistakes like "https://x/y" + "admin" → "https://x/yadmin".
    // Errors here (e.g., non-http scheme) turn into `Err(DirustError::InvalidBaseUrl)`.
    let base: String = url::normalize_base(&args.base)?;

    // Build a single reusable HTTP client. This client is cheap to clone and will
    // share connection pools among tasks. We set:
    //   - a custom User-Agent (helps identify the tool in logs)
    //   - redirect policy = none (we want to *see* 30x + Location headers)
    //   - a per-request timeout derived from CLI (to avoid hung sockets)
    let client: Client = Client::builder()
        .user_agent("dirust/0.1.1")
        .redirect(reqwest::redirect::Policy::none())
        .timeout(args.request_timeout())
        .build()?; // Any reqwest build error becomes `DirustError::Http` via `From`

    // Kick off the scan orchestration. This will:
    //   - read the wordlist,
    //   - expand targets (base + word [+ ext]),
    //   - run bounded-concurrency probes,
    //   - and print "interesting" results (200/301/302/401/403).
    //
    // Any error encountered inside (I/O, HTTP, task join) bubbles up as `Err(DirustError)`.
    scanner::scan(&client, &base, &args).await
}
