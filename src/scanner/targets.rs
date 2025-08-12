pub fn build_targets(base: &str, words: &[String], exts: &[String]) -> Vec<String> {
    // Pre-size the vector to reduce reallocations.
    let multiplier: usize = if exts.is_empty() { 1 } else { 1 + exts.len() };
    let capacity: usize = words.len() * multiplier;
    let mut targets: Vec<String> = Vec::with_capacity(capacity);

    for raw in words {
        // 1) Clean the input word
        let trimmed: &str = raw.trim();
        let cleaned: &str = trimmed.trim_start_matches('/');

        println!("Processing word: {}", cleaned);

        if cleaned.is_empty() {
            // Skip empty entries
            continue;
        }

        // 2) Classify the entry
        let contains_slash: bool = cleaned.contains('/');
        let ends_with_slash: bool = cleaned.ends_with('/');
        let has_dot: bool = cleaned.contains('.');

        // If it contains any slash, we consider it a directory-like path.
        // Also, if it ends with '/', it's *definitely* a directory.
        let treat_as_directory: bool = contains_slash || ends_with_slash;

        // 3) Always include the base + cleaned path as-is
        let as_is: String = format!("{}{}", base, cleaned);
        println!("{}", as_is);
        targets.push(as_is);

        // 4) Only append extensions when:
        //    - NOT a directory-like path
        //    - AND the word does NOT already include a dot (no existing extension)
        if !treat_as_directory && !has_dot {
            for ext in exts {
                let with_ext: String = format!("{}{}{}", base, cleaned, ext);
                println!("{}", with_ext);
                targets.push(with_ext);
            }
        }
    }

    targets
}
