use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
    /// Base URL (e.g., https://example.com/ or https://example.com/app/)
    pub base: String,

    /// Path to wordlist file
    #[arg(short, long)]
    pub wordlist: String,

    /// Concurrency (requests in flight)
    #[arg(short, long, default_value_t = 50)]
    pub concurrency: usize,

    /// Use GET instead of HEAD
    #[arg(long, default_value_t = false)]
    pub get: bool,

    /// Timeout per request in seconds
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Extra extensions to try (comma-separated, e.g. "php,html,txt")
    #[arg(long, default_value = "")]
    pub exts: String,
}

impl Args {
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }
    pub fn parse_exts(&self) -> Vec<String> {
        self.exts
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| format!(".{}", s.trim().trim_start_matches('.')))
            .collect()
    }
}
