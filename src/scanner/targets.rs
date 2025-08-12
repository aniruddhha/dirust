pub fn build_targets(base: &str, words: &[String], exts: &[String]) -> Vec<String> {
    let capacity = words.len() * core::cmp::max(exts.len(), 1);
    let mut targets: Vec<String> = Vec::with_capacity(capacity);

    for raw in words {
        let cleaned = raw.trim().trim_start_matches('/');

        if cleaned.is_empty() {
            continue;
        }

        // Word as-is
        targets.push(format!("{}{}", base, cleaned));

        // Word with extensions
        for ext in exts {
            targets.push(format!("{}{}{}", base, cleaned, ext));
        }
    }

    targets
}