# Dirust
Fast, async directory brute-forcer written in Rust.

## Features
- Async HTTP with Tokio + Reqwest
- Bounded concurrency
- Optional extensions
- Status code filtering

## Usage
```bash
cargo run --release -- https://target.com/ -w wordlist.txt