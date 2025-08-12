/// Build a list of absolute URLs to probe, based on:
///   - `base`: normalized base URL (must end with '/')
///   - `words`: entries from the wordlist (e.g., "admin", "admin/", "readme.txt")
///   - `exts`: extra extensions to try (e.g., [".php", ".html", ".txt"])
///
/// Rules:
///   1) Always include the "as-is" path: base + cleaned word.
///   2) If the word represents a directory (contains '/' anywhere OR ends with '/'),
///      DO NOT append extra extensions.
///   3) If the word already has a dot (e.g., "readme.txt"), treat it as a file that
///      already has an extension — DO NOT append extra extensions.
///   4) Only when the word is a "plain name" (no '/' and no '.'), append all extra extensions.
pub fn build_targets(base: &str, words: &[String], exts: &[String]) -> Vec<String> {
    // Pre-calculate capacity to reduce re-allocations:
    // - If there are no extensions, we add exactly 1 target per word (the as-is URL).
    // - If there are N extensions, we add up to (1 + N) targets per word (as-is + each ext).
    let per_word_estimate: usize = if exts.is_empty() { 1 } else { 1 + exts.len() };
    let capacity: usize = words.len() * per_word_estimate;

    // Pre-allocate the output vector with the estimated capacity.
    let mut targets: Vec<String> = Vec::with_capacity(capacity);

    // Iterate every word from the wordlist.
    for raw in words {
        // 1) Normalize the input word:
        //    - Trim whitespace at both ends.
        //    - Remove a leading '/' if present so we don't accidentally double the slash (`base//word`).
        let trimmed: &str = raw.trim();
        let cleaned: &str = trimmed.trim_start_matches('/');

        // Optional: progress logging to understand what we are processing.
        println!("Processing word: {}", cleaned);

        // Skip empty lines or lines that become empty after trimming.
        if cleaned.is_empty() {
            continue;
        }

        // 2) Classify what this entry looks like.
        // `contains_slash`: the word has a '/' anywhere (e.g., "admin/", "admin/panel", "api/v1/").
        // `ends_with_slash`: a strong signal of a directory (e.g., "admin/").
        // `has_dot`: there is at least one dot; often implies an existing extension (e.g., "readme.txt").
        let contains_slash: bool = cleaned.contains('/');
        let ends_with_slash: bool = cleaned.ends_with('/');
        let has_dot: bool = cleaned.contains('.');

        // Treat the entry as directory-like if:
        //  - it contains any slash (intermediate subpaths), OR
        //  - it ends with a slash (explicitly a directory).
        let treat_as_directory: bool = contains_slash || ends_with_slash;

        // 3) Always include the "as-is" URL (base + cleaned).
        //    This covers:
        //    - plain files ("readme.txt" -> ".../readme.txt")
        //    - plain names ("admin" -> ".../admin")
        //    - directories ("admin/" -> ".../admin/")
        let as_is_url: String = format!("{}{}", base, cleaned);
        println!("{}", as_is_url);
        targets.push(as_is_url);

        // 4) Only append extensions when the entry is a simple "name" (no slashes, no dots).
        //    Examples where we DO append:
        //      "admin"   -> ".../admin.php", ".../admin.html", ...
        //      "status"  -> ".../status.php", ...
        //    Examples where we DO NOT append:
        //      "admin/"  -> directory -> skip extensions
        //      "api/v1"  -> has slash -> treat as directory-like -> skip extensions
        //      "readme.txt" -> already has dot -> skip extensions
        if !treat_as_directory && !has_dot {
            // Append each configured extension to the base + cleaned word.
            for ext in exts {
                let with_ext_url: String = format!("{}{}{}", base, cleaned, ext);
                println!("{}", with_ext_url);
                targets.push(with_ext_url);
            }
        }
    }

    // Return the complete list of targets to probe.
    targets
}
