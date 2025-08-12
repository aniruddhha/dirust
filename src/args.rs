//! src/args.rs  (rename to `src/arg.rs` if your project uses that filename)
//!
//! Purpose:
//!   Define the command-line interface (CLI) for Dirust using `clap`'s derive API.
//!   This struct describes all the flags and positional arguments the binary accepts,
//!   and `Args::parse()` will populate it from `std::env::args()` at runtime.
//!
//! Notes:
//!   - We keep the code explicit and add detailed comments for learning clarity.
//!   - No `anyhow` is used anywhere in the project, per your preference.

use clap::Parser;
use std::time::Duration;

/// Top-level CLI configuration for Dirust.
///
/// The `#[derive(Parser)]` attribute instructs `clap` to generate the argument
/// parsing logic for this struct. The field names and `#[arg(...)]` attributes
/// become your command-line flags and positional arguments.
///
/// `author`, `version`, and `about` are used by `--help` and `--version`.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
    /// Base URL to scan (must start with http:// or https://).
    ///
    /// This is a *positional* argument — no flag is required. Example:
    ///     dirust https://example.com/ -w words.txt
    ///
    /// The program will later normalize this to ensure it ends with a trailing `/`.
    pub base: String,

    /// Path to the wordlist file (e.g., rockyou-like list of endpoints).
    ///
    /// Short form:  -w <PATH>
    /// Long form:   --wordlist <PATH>
    #[arg(short, long)]
    pub wordlist: String,

    /// Maximum number of in-flight requests (concurrency cap).
    ///
    /// Short form:  -c <N>
    /// Long form:   --concurrency <N>
    ///
    /// This value is enforced with a semaphore in the scanner; it prevents
    /// overwhelming the target or your own machine.
    #[arg(short, long, default_value_t = 50)]
    pub concurrency: usize,

    /// If present, use GET for all requests instead of HEAD.
    ///
    /// By default the scanner prefers HEAD (faster, typically no body).
    /// Some servers misbehave on HEAD; this flag forces GET.
    ///
    /// Long form only (boolean flag):
    ///     --get
    #[arg(long, default_value_t = false)]
    pub get: bool,

    /// Per-request timeout in seconds.
    ///
    /// Long form:
    ///     --timeout <SECS>
    ///
    /// The value is converted to `std::time::Duration` by `request_timeout()`.
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Extra extensions to try for plain names (comma-separated).
    ///
    /// Example:
    ///     --exts php,html,txt
    ///
    /// Behavior:
    ///   - For a word like "admin", the scanner will also try "admin.php", "admin.html", "admin.txt".
    ///   - For entries that already look like files (contain a dot, e.g., "readme.txt"),
    ///     *no* extra extensions are appended.
    ///   - For entries that look like directories (contain '/' anywhere or end with '/'),
    ///     *no* extra extensions are appended.
    #[arg(long, default_value = "")]
    pub exts: String,
}

impl Args {
    /// Convert the numeric `timeout` into a `Duration`.
    ///
    /// We keep this as a method to make call sites (client builder) explicit and readable.
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }

    /// Parse the comma-separated `exts` string into a normalized list of extensions.
    ///
    /// Rules:
    ///   - Split by comma.
    ///   - Trim whitespace around each token.
    ///   - Ignore empty tokens (e.g., trailing comma).
    ///   - Ensure each extension starts with exactly one dot:
    ///       "php"   -> ".php"
    ///       ".html" -> ".html"
    ///       ""      -> (ignored)
    ///
    /// Returns:
    ///   A `Vec<String>` such as: vec![".php", ".html", ".txt"]
    pub fn parse_exts(&self) -> Vec<String> {
        // We will build the result step by step (no iterator shortcuts),
        // adding comments to make each step explicit.
        let mut out: Vec<String> = Vec::new();

        // Split by comma to get raw tokens. An empty `self.exts` becomes a single empty token,
        // which we will filter out below.
        for raw_token in self.exts.split(',') {
            // Remove surrounding whitespace (e.g., "  php  " -> "php").
            let trimmed = raw_token.trim();

            // Skip tokens that are empty after trimming.
            if trimmed.is_empty() {
                continue;
            }

            // If the token already starts with a dot, keep it as-is.
            // If it does not, prepend a dot to normalize it.
            //
            // Using `trim_start_matches('.')` avoids accidental double-dots like "..php".
            // Example:
            //   trimmed = "php"    -> no_dot = "php"    -> normalized = ".php"
            //   trimmed = ".html"  -> no_dot = "html"   -> normalized = ".html"
            let no_dot = trimmed.trim_start_matches('.');
            let normalized = format!(".{}", no_dot);

            // Push the normalized extension into the result list.
            out.push(normalized);
        }

        // Return the fully built list of extensions (possibly empty).
        out
    }
}
