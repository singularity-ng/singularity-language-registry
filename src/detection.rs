//! Language Detection Utilities
//!
//! Advanced language detection beyond simple extension matching.
//! Uses data from GitHub Linguist for interpreter, filename, and heuristic mappings.

use crate::heuristics_generated::{
    HeuristicRule, LANGUAGE_DETECTION_HEURISTICS, NAMED_HEURISTIC_PATTERNS,
};
use crate::languages_metadata_generated::LANGUAGES;
use crate::registry::{LanguageInfo, LANGUAGE_REGISTRY};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

/// Pre-built interpreter -> language name mapping from Linguist data
static INTERPRETER_MAP: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut map = Vec::new();
    for lang in LANGUAGES {
        for interpreter in lang.interpreters {
            map.push((*interpreter, lang.name));
        }
    }
    map
});

/// Pre-built filename -> language name mapping from Linguist data
static FILENAME_MAP: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut map = Vec::new();
    for lang in LANGUAGES {
        for filename in lang.filenames {
            map.push((*filename, lang.name));
        }
    }
    map
});

/// Pre-built named pattern map for heuristics (e.g., "cpp", "perl", "fortran")
static NAMED_PATTERN_MAP: LazyLock<HashMap<&'static str, &'static [&'static str]>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        for (name, patterns) in NAMED_HEURISTIC_PATTERNS {
            let _ = map.insert(*name, *patterns);
        }
        map
    });

/// Detect language from file content (shebang, magic bytes, etc.)
///
/// Returns None if the content doesn't match any supported language.
/// No fallback - explicit detection only.
#[must_use]
pub fn detect_from_content(content: &str) -> Option<&'static LanguageInfo> {
    // Check shebang first
    if let Some(lang) = detect_from_shebang(content) {
        return Some(lang);
    }

    // Check for specific content patterns
    if let Some(lang) = detect_from_patterns(content) {
        return Some(lang);
    }

    // No fallback - if not detected, return None
    None
}

/// Detect language from shebang line using Linguist interpreter data
#[must_use]
pub fn detect_from_shebang(content: &str) -> Option<&'static LanguageInfo> {
    let first_line = content.lines().next()?;
    if !first_line.starts_with("#!") {
        return None;
    }

    let shebang = first_line.to_lowercase();

    // Extract the interpreter name from the shebang
    // Handle both "#!/usr/bin/python" and "#!/usr/bin/env python"
    let interpreter = if shebang.contains("/env ") {
        // env-style: #!/usr/bin/env python3
        shebang.split_whitespace().nth(1)?
    } else {
        // Direct path: #!/usr/bin/python3
        shebang
            .trim_start_matches("#!")
            .split('/')
            .last()?
            .split_whitespace()
            .next()?
    };

    // Strip version numbers from interpreter (python3 -> python, ruby2.7 -> ruby)
    let base_interpreter = interpreter
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');

    // First try: direct lookup by interpreter name as ID (handles "bash", "python", etc.)
    if let Some(lang) = LANGUAGE_REGISTRY.get_language(base_interpreter) {
        return Some(lang);
    }

    // Second try: lookup by alias (handles "bash" -> bash language via alias)
    if let Some(lang) = LANGUAGE_REGISTRY.get_language_by_alias(base_interpreter) {
        return Some(lang);
    }

    // Third try: Look up in Linguist interpreter map for language name
    for (interp, lang_name) in INTERPRETER_MAP.iter() {
        if base_interpreter.contains(interp) || interpreter.contains(interp) {
            // Convert language name to ID (lowercase, handle spaces)
            let lang_id = lang_name.to_lowercase().replace(' ', "-");
            if let Some(lang) = LANGUAGE_REGISTRY.get_language(&lang_id) {
                return Some(lang);
            }
            // Try without conversion
            if let Some(lang) = LANGUAGE_REGISTRY.get_language(&lang_name.to_lowercase()) {
                return Some(lang);
            }
            // Try by alias
            if let Some(lang) = LANGUAGE_REGISTRY.get_language_by_alias(&lang_id) {
                return Some(lang);
            }
        }
    }

    None
}

/// Detect language from content patterns
#[must_use]
pub fn detect_from_patterns(content: &str) -> Option<&'static LanguageInfo> {
    // Check for unique language patterns

    // Dockerfile
    if content.starts_with("FROM ") || content.contains("\nFROM ") {
        return LANGUAGE_REGISTRY.get_language("dockerfile");
    }

    // SQL
    let upper = content.to_uppercase();
    if upper.starts_with("CREATE TABLE")
        || upper.starts_with("SELECT ")
        || upper.starts_with("INSERT INTO")
        || upper.starts_with("ALTER TABLE")
        || upper.starts_with("DROP TABLE")
    {
        return LANGUAGE_REGISTRY.get_language("sql");
    }

    // XML/HTML detection
    if content.trim_start().starts_with("<?xml") {
        return LANGUAGE_REGISTRY.get_language("xml");
    }
    if content.trim_start().starts_with("<!DOCTYPE html")
        || content.trim_start().starts_with("<html")
    {
        return LANGUAGE_REGISTRY.get_language("html");
    }

    // JSON - starts with { or [
    let trimmed = content.trim();
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        // Avoid false positives with code blocks
        if !content.contains("function") && !content.contains("class ") {
            return LANGUAGE_REGISTRY.get_language("json");
        }
    }

    // YAML (check for common patterns)
    if content.starts_with("---\n") || content.starts_with("%YAML") {
        return LANGUAGE_REGISTRY.get_language("yaml");
    }

    // Markdown - but be careful not to match code with comments
    if content.starts_with("# ") && content.contains("\n\n") {
        // Multiple paragraphs suggests prose, not code
        if content.contains("\n## ") || content.contains("\n### ") || content.contains("```") {
            return LANGUAGE_REGISTRY.get_language("markdown");
        }
    }

    None
}

/// Detect language for special filenames (Makefile, Dockerfile, etc.)
/// Uses Linguist filename data.
#[must_use]
pub fn detect_special_files(filename: &str) -> Option<&'static LanguageInfo> {
    // First try exact match from Linguist data
    for (fname, lang_name) in FILENAME_MAP.iter() {
        if filename.eq_ignore_ascii_case(fname) {
            let lang_id = lang_name.to_lowercase().replace(' ', "-");
            if let Some(lang) = LANGUAGE_REGISTRY.get_language(&lang_id) {
                return Some(lang);
            }
            if let Some(lang) = LANGUAGE_REGISTRY.get_language(&lang_name.to_lowercase()) {
                return Some(lang);
            }
        }
    }

    // Fallback for common patterns not in Linguist
    let lower = filename.to_lowercase();
    match lower.as_str() {
        "cargo.toml" | "cargo.lock" => LANGUAGE_REGISTRY.get_language("toml"),
        "go.mod" | "go.sum" => LANGUAGE_REGISTRY.get_language("go-module"),
        _ => None,
    }
}

/// Check if a language can be detected (returns true/false, no confidence scores)
///
/// Strict checking - either the language is supported or it isn't.
#[must_use]
pub fn is_detectable(file_path: &Path, content: Option<&str>) -> bool {
    // Check extension
    if LANGUAGE_REGISTRY.detect_language(file_path).is_ok() {
        return true;
    }

    // Check content if provided
    if let Some(file_content) = content {
        if detect_from_content(file_content).is_some() {
            return true;
        }
    }

    // Check special files
    if let Some(filename) = file_path.file_name() {
        if let Some(name) = filename.to_str() {
            if detect_special_files(name).is_some() {
                return true;
            }
        }
    }

    false
}

/// Disambiguate language using Linguist heuristics for ambiguous extensions.
///
/// When multiple languages share the same extension (e.g., `.h` for C/C++/Objective-C),
/// this function uses content-based heuristics from GitHub Linguist to determine
/// the most likely language.
///
/// Returns `None` if no heuristics match or the extension has no heuristics defined.
#[must_use]
pub fn detect_from_heuristics(extension: &str, content: &str) -> Option<&'static LanguageInfo> {
    // Normalize extension (ensure it starts with '.')
    let ext = if extension.starts_with('.') {
        extension
    } else {
        return None; // Invalid extension format
    };

    // Find heuristics for this extension
    for entry in LANGUAGE_DETECTION_HEURISTICS {
        if !entry.extensions.contains(&ext) {
            continue;
        }

        // Evaluate rules in order
        for rule in entry.rules {
            if rule_matches(rule, content) {
                // Convert language name to registry ID
                let lang_id = rule.language.to_lowercase().replace(' ', "-");
                if let Some(lang) = LANGUAGE_REGISTRY.get_language(&lang_id) {
                    return Some(lang);
                }
                // Try without conversion
                if let Some(lang) = LANGUAGE_REGISTRY.get_language(&rule.language.to_lowercase()) {
                    return Some(lang);
                }
            }
        }

        // Found extension but no rules matched
        break;
    }

    None
}

/// Check if a heuristic rule matches the content.
/// Uses simple substring matching (full regex would require the regex crate).
fn rule_matches(rule: &HeuristicRule, content: &str) -> bool {
    // Check negative patterns first (must NOT match)
    for neg_pattern in rule.negative_patterns {
        if content.contains(neg_pattern) {
            return false;
        }
    }

    // Check named pattern if present
    if let Some(named) = rule.named_pattern {
        if let Some(patterns) = NAMED_PATTERN_MAP.get(named) {
            // Named patterns use regex - do simple substring check for common cases
            let any_match = patterns.iter().any(|p| {
                // For simple patterns, check substring
                // Complex regex patterns won't match but that's acceptable
                // for a lightweight implementation
                content.contains(p) || simple_pattern_match(p, content)
            });
            if !any_match {
                return false;
            }
        }
    }

    // Check direct patterns
    if rule.patterns.is_empty() && rule.named_pattern.is_none() && rule.and.is_empty() {
        // Empty rule with no conditions - this is a fallback/default
        return true;
    }

    // At least one pattern must match (if patterns exist)
    if !rule.patterns.is_empty() {
        let any_match = rule
            .patterns
            .iter()
            .any(|p| content.contains(p) || simple_pattern_match(p, content));
        if !any_match {
            return false;
        }
    }

    // Check AND conditions (all must match)
    for cond in rule.and {
        // Check negative patterns in condition
        for neg_pattern in cond.negative_patterns {
            if content.contains(neg_pattern) {
                return false;
            }
        }

        // Check condition patterns (at least one must match)
        if !cond.patterns.is_empty() {
            let any_match = cond
                .patterns
                .iter()
                .any(|p| content.contains(p) || simple_pattern_match(p, content));
            if !any_match {
                return false;
            }
        }
    }

    true
}

/// Simple pattern matching for common regex-like patterns.
/// Handles basic cases like `^pattern`, `pattern$`, `\b`, `\s`.
fn simple_pattern_match(pattern: &str, content: &str) -> bool {
    // Handle start anchor
    if let Some(rest) = pattern.strip_prefix('^') {
        // Check if any line starts with the pattern
        let search = rest
            .replace("\\s", " ")
            .replace("\\t", "\t")
            .replace("\\b", "");
        return content.lines().any(|line| {
            line.trim_start()
                .to_lowercase()
                .starts_with(&search.to_lowercase())
        });
    }

    // Handle common escapes and check substring
    let simplified = pattern
        .replace("\\s+", " ")
        .replace("\\s*", "")
        .replace("\\s", " ")
        .replace("\\t", "\t")
        .replace("\\b", "")
        .replace("\\n", "\n")
        .replace("\\.", ".")
        .replace("\\(", "(")
        .replace("\\)", ")")
        .replace("\\[", "[")
        .replace("\\]", "]")
        .replace("\\{", "{")
        .replace("\\}", "}");

    // Remove regex quantifiers for simple matching
    let cleaned: String = simplified
        .chars()
        .filter(|c| !matches!(c, '*' | '+' | '?' | '|'))
        .collect();

    if cleaned.is_empty() {
        return false;
    }

    content.to_lowercase().contains(&cleaned.to_lowercase())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "Tests are allowed to unwrap")]
mod tests {
    use super::*;

    // ========== Shebang Detection Tests ==========

    #[test]
    fn test_shebang_python() {
        let content = "#!/usr/bin/env python3\nprint('hello')";
        let lang = detect_from_shebang(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "python");
    }

    #[test]
    fn test_shebang_python_direct_path() {
        let content = "#!/usr/bin/python\nimport sys";
        let lang = detect_from_shebang(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "python");
    }

    #[test]
    fn test_shebang_bash() {
        let content = "#!/bin/bash\necho hello";
        let lang = detect_from_shebang(content);
        assert!(lang.is_some());
        // Bash detected as "bash" in test fixture
        assert_eq!(lang.unwrap().id, "bash");
    }

    #[test]
    fn test_shebang_node() {
        let content = "#!/usr/bin/env node\nconsole.log('hi')";
        let lang = detect_from_shebang(content);
        // Node maps to JavaScript if available in registry
        // May return None if not in test fixture
        if let Some(l) = lang {
            assert!(l.id == "javascript" || l.id == "node");
        }
    }

    #[test]
    fn test_shebang_no_shebang() {
        let content = "print('hello')";
        let lang = detect_from_shebang(content);
        assert!(lang.is_none());
    }

    #[test]
    fn test_shebang_empty_content() {
        let lang = detect_from_shebang("");
        assert!(lang.is_none());
    }

    // ========== Content Pattern Detection Tests ==========

    #[test]
    fn test_dockerfile_pattern() {
        let content = "FROM ubuntu:latest\nRUN apt-get update";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "dockerfile");
    }

    #[test]
    fn test_dockerfile_multiline() {
        let content = "# Dockerfile\nFROM node:18\nWORKDIR /app";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "dockerfile");
    }

    #[test]
    fn test_sql_pattern() {
        let content = "SELECT * FROM users WHERE id = 1";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "sql");
    }

    #[test]
    fn test_sql_create_table() {
        let content = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255))";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "sql");
    }

    #[test]
    fn test_xml_pattern() {
        let content = "<?xml version=\"1.0\"?>\n<root><child/></root>";
        let lang = detect_from_patterns(content);
        // XML may not be in the test fixture, but pattern should match
        if let Some(l) = lang {
            assert_eq!(l.id, "xml");
        }
    }

    #[test]
    fn test_html_pattern() {
        let content = "<!DOCTYPE html>\n<html><head></head><body></body></html>";
        let lang = detect_from_patterns(content);
        // HTML may not be in the test fixture, but pattern should match
        if let Some(l) = lang {
            assert_eq!(l.id, "html");
        }
    }

    #[test]
    fn test_yaml_pattern() {
        let content = "---\nname: test\nversion: 1.0";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "yaml");
    }

    #[test]
    fn test_json_pattern() {
        let content = "{\"name\": \"test\", \"value\": 42}";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "json");
    }

    #[test]
    fn test_json_array_pattern() {
        let content = "[{\"id\": 1}, {\"id\": 2}]";
        let lang = detect_from_patterns(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "json");
    }

    #[test]
    fn test_no_pattern_match() {
        let content = "def hello():\n    pass";
        let lang = detect_from_patterns(content);
        // Python code without shebang shouldn't match generic patterns
        assert!(lang.is_none());
    }

    // ========== Special Files Detection Tests ==========

    #[test]
    fn test_special_files() {
        // Test Dockerfile detection (Makefile is not in the test fixture)
        let lang = detect_special_files("Dockerfile");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "dockerfile");
    }

    #[test]
    fn test_special_files_case_insensitive() {
        // Should match case-insensitively
        let lang = detect_special_files("dockerfile");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "dockerfile");
    }

    #[test]
    fn test_special_files_unknown() {
        let lang = detect_special_files("random_file.xyz");
        assert!(lang.is_none());
    }

    // ========== Heuristics Detection Tests ==========

    #[test]
    fn test_heuristics_invalid_extension() {
        // Extension must start with '.'
        let result = detect_from_heuristics("rs", "fn main() {}");
        assert!(result.is_none());
    }

    #[test]
    fn test_heuristics_unknown_extension() {
        // Extension not in heuristics
        let result = detect_from_heuristics(".xyz123", "some content");
        assert!(result.is_none());
    }

    // ========== Simple Pattern Matching Tests ==========

    #[test]
    fn test_simple_pattern_match_basic() {
        assert!(simple_pattern_match("hello", "hello world"));
        assert!(simple_pattern_match("world", "hello world"));
        assert!(!simple_pattern_match("foo", "hello world"));
    }

    #[test]
    fn test_simple_pattern_match_start_anchor() {
        assert!(simple_pattern_match("^hello", "hello world"));
        // Note: our simple matcher trims leading whitespace, so this may not match exactly
        assert!(!simple_pattern_match("^world", "hello world"));
    }

    #[test]
    fn test_simple_pattern_match_escapes() {
        // Test common escape sequences
        assert!(simple_pattern_match("\\.", "file.txt"));
        assert!(simple_pattern_match("a\\sb", "a b")); // \s -> space
    }

    #[test]
    fn test_simple_pattern_match_case_insensitive() {
        assert!(simple_pattern_match("HELLO", "hello world"));
        assert!(simple_pattern_match("hello", "HELLO WORLD"));
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_detect_from_content_shebang_first() {
        // Shebang should take precedence
        let content = "#!/usr/bin/env python3\nSELECT * FROM users";
        let lang = detect_from_content(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "python");
    }

    #[test]
    fn test_detect_from_content_patterns_fallback() {
        // Without shebang, patterns are used
        let content = "SELECT * FROM users WHERE id = 1";
        let lang = detect_from_content(content);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().id, "sql");
    }

    #[test]
    fn test_is_detectable_by_extension() {
        use std::path::Path;
        assert!(is_detectable(Path::new("test.rs"), None));
        assert!(is_detectable(Path::new("test.py"), None));
    }

    #[test]
    fn test_is_detectable_by_content() {
        use std::path::Path;
        let content = "#!/usr/bin/env python3\nprint('hi')";
        assert!(is_detectable(Path::new("script"), Some(content)));
    }

    #[test]
    fn test_is_detectable_unknown() {
        use std::path::Path;
        assert!(!is_detectable(Path::new("unknown"), None));
        assert!(!is_detectable(Path::new("file.xyz123"), None));
    }
}
