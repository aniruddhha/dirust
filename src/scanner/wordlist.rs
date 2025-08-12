use crate::error::DirustError;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn read_wordlist(path: &str) -> Result<Vec<String>, DirustError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut out: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with('#') {
                    continue;
                }
                out.push(trimmed);
            }
            Err(e) => {
                // Stop on the first I/O error
                return Err(DirustError::from(e));
            }
        }
    }

    Ok(out)
}