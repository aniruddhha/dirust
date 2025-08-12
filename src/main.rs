mod args;
mod error;
mod scanner;
mod url; 

use args::Args;
use clap::Parser;
use error::DirustError;
use reqwest::Client;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), DirustError> {
    let args = Args::parse();

    let base = url::normalize_base(&args.base)?;
    let client = Client::builder()
        .user_agent("dirust/0.1.1")
        .redirect(reqwest::redirect::Policy::none())
        .timeout(args.request_timeout())
        .build()?;

    scanner::scan(&client, &base, &args).await
}