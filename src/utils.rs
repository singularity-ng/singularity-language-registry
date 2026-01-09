//! Utility functions for language operations

use crate::registry::{LanguageCapability, LanguageInfo, LANGUAGE_REGISTRY};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

/// Get all languages grouped by family
pub fn languages_by_families() -> HashMap<String, Vec<&'static LanguageInfo>> {
    let mut families: HashMap<String, Vec<&'static LanguageInfo>> = HashMap::new();

    for lang in LANGUAGE_REGISTRY.supported_languages() {
        if let Some(ref family) = lang.family {
            families.entry(family.clone()).or_default().push(lang);
        } else {
            families.entry("Other".to_owned()).or_default().push(lang);
        }
    }

    families
}

/// Get language statistics
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct LanguageStats {
    pub total_languages: usize,
    pub rca_supported: usize,
    pub ast_grep_supported: usize,
    pub compiled_languages: usize,
    pub interpreted_languages: usize,
    pub families: usize,
}

impl LanguageStats {
    pub fn calculate() -> Self {
        let all = LANGUAGE_REGISTRY.supported_languages();
        let families = languages_by_families();

        Self {
            total_languages: all.len(),
            rca_supported: all
                .iter()
                .filter(|l| l.rca_supported.load(Ordering::Relaxed))
                .count(),
            ast_grep_supported: all
                .iter()
                .filter(|l| l.ast_grep_supported.load(Ordering::Relaxed))
                .count(),
            compiled_languages: all.iter().filter(|l| l.is_compiled).count(),
            interpreted_languages: all.iter().filter(|l| !l.is_compiled).count(),
            families: families.len(),
        }
    }
}

/// Check if two languages are in the same family
pub fn same_family(lang1: &str, lang2: &str) -> bool {
    let Some(l1) = LANGUAGE_REGISTRY.get_language(lang1) else {
        return false;
    };

    let Some(l2) = LANGUAGE_REGISTRY.get_language(lang2) else {
        return false;
    };

    match (l1.family.as_ref(), l2.family.as_ref()) {
        (Some(f1), Some(f2)) => f1 == f2,
        _ => false,
    }
}

/// Get recommended linters for a language.
///
/// Returns linters from the language metadata if available,
/// otherwise falls back to a hardcoded list for common languages.
#[must_use]
pub fn recommended_linters(language: &str) -> Vec<String> {
    // Try to get from language metadata first
    if let Some(lang) = LANGUAGE_REGISTRY.get_language(language) {
        if !lang.linters.is_empty() {
            return lang.linters.clone();
        }
    }

    // Fallback to hardcoded defaults for languages without metadata
    match language {
        "rust" => vec!["clippy".into(), "rustfmt".into()],
        "javascript" | "typescript" => vec!["eslint".into(), "prettier".into()],
        "python" => vec!["pylint".into(), "black".into(), "mypy".into()],
        "go" => vec!["golangci-lint".into(), "gofmt".into()],
        "elixir" => vec!["credo".into(), "mix format".into()],
        "erlang" => vec!["dialyzer".into()],
        "java" => vec!["spotbugs".into(), "checkstyle".into()],
        "c" | "cpp" => vec!["clang-tidy".into(), "cppcheck".into()],
        "csharp" => vec!["roslyn".into(), "sonarqube".into()],
        "ruby" => vec!["rubocop".into()],
        _ => vec![],
    }
}

/// Get common file patterns for a language (beyond extensions).
///
/// Returns file patterns from the language metadata if available,
/// otherwise falls back to a hardcoded list for common languages.
#[must_use]
pub fn file_patterns(language: &str) -> Vec<String> {
    // Try to get from language metadata first
    if let Some(lang) = LANGUAGE_REGISTRY.get_language(language) {
        if !lang.file_patterns.is_empty() {
            return lang.file_patterns.clone();
        }
    }

    // Fallback to hardcoded defaults for languages without metadata
    match language {
        "rust" => vec!["Cargo.toml".into(), "Cargo.lock".into(), "build.rs".into()],
        "javascript" | "typescript" => {
            vec![
                "package.json".into(),
                "tsconfig.json".into(),
                "webpack.config.js".into(),
            ]
        }
        "python" => vec![
            "requirements.txt".into(),
            "setup.py".into(),
            "pyproject.toml".into(),
            "Pipfile".into(),
        ],
        "go" => vec!["go.mod".into(), "go.sum".into()],
        "elixir" => vec!["mix.exs".into(), "mix.lock".into()],
        "erlang" => vec!["rebar.config".into(), "erlang.mk".into()],
        "java" => vec!["pom.xml".into(), "build.gradle".into()],
        "ruby" => vec!["Gemfile".into(), "Gemfile.lock".into(), "Rakefile".into()],
        _ => vec![],
    }
}

/// Deprecated: Use [`LanguageCapability`] instead.
///
/// This type alias exists for backwards compatibility.
#[deprecated(since = "0.3.0", note = "Use LanguageCapability instead")]
pub type AnalysisFeature = LanguageCapability;

/// Check if a language supports a specific capability.
///
/// For explicit capabilities (RCA, ASTGrep, Linting, Parsing, CodeEngine),
/// this checks the capability bitmask. For implicit capabilities (TreeSitter,
/// Complexity, Security, Performance), this uses language properties.
pub fn supports_feature(language: &str, capability: LanguageCapability) -> bool {
    let Some(lang) = LANGUAGE_REGISTRY.get_language(language) else {
        return false;
    };

    match capability {
        // Explicit capabilities - check bitmask or atomic bool
        LanguageCapability::RCA => lang.rca_supported.load(Ordering::Relaxed),
        LanguageCapability::ASTGrep => lang.ast_grep_supported.load(Ordering::Relaxed),
        LanguageCapability::Linting
        | LanguageCapability::Parsing
        | LanguageCapability::CodeEngine => lang.has_capability(capability),

        // Implicit capabilities - determined by language properties
        LanguageCapability::TreeSitter => lang.tree_sitter_language.is_some(),
        LanguageCapability::Complexity => {
            // Most compiled languages support complexity analysis
            lang.is_compiled || matches!(language, "python" | "javascript" | "typescript")
        }
        LanguageCapability::Security => {
            // Languages commonly used in web/system development
            matches!(
                language,
                "rust"
                    | "go"
                    | "python"
                    | "javascript"
                    | "typescript"
                    | "java"
                    | "csharp"
                    | "c"
                    | "cpp"
                    | "ruby"
                    | "elixir"
            )
        }
        LanguageCapability::Performance => {
            // Compiled languages + performance-critical interpreted ones
            lang.is_compiled || matches!(language, "python" | "javascript" | "typescript")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_linters_from_metadata() {
        // Elixir has linters in metadata
        let linters = recommended_linters("elixir");
        assert!(!linters.is_empty(), "elixir should have linters");
        assert!(
            linters.iter().any(|l| l.contains("credo")),
            "elixir linters should include credo"
        );
    }

    #[test]
    fn test_recommended_linters_fallback() {
        // Go uses hardcoded fallback (no linters in test fixture)
        let linters = recommended_linters("go");
        assert!(
            linters.iter().any(|l| l.contains("golangci-lint")),
            "go should fallback to hardcoded linters"
        );
    }

    #[test]
    fn test_recommended_linters_unknown() {
        let linters = recommended_linters("nonexistent-language");
        assert!(linters.is_empty(), "unknown language should return empty");
    }

    #[test]
    fn test_file_patterns_from_metadata() {
        // Rust has file_patterns in metadata
        let patterns = file_patterns("rust");
        assert!(!patterns.is_empty(), "rust should have file patterns");
        assert!(
            patterns.iter().any(|p| p.contains("Cargo.toml")),
            "rust patterns should include Cargo.toml"
        );
    }

    #[test]
    fn test_file_patterns_fallback() {
        // Python uses hardcoded fallback (no file_patterns in test fixture)
        let patterns = file_patterns("python");
        assert!(
            patterns.iter().any(|p| p.contains("requirements.txt")),
            "python should fallback to hardcoded patterns"
        );
    }

    #[test]
    fn test_file_patterns_unknown() {
        let patterns = file_patterns("nonexistent-language");
        assert!(patterns.is_empty(), "unknown language should return empty");
    }

    #[test]
    fn test_languages_by_families() {
        let families = languages_by_families();
        assert!(!families.is_empty(), "should have some families");

        // Check BEAM family exists and has expected languages
        if let Some(beam) = families.get("BEAM") {
            assert!(
                beam.iter().any(|l| l.id == "elixir"),
                "BEAM should include elixir"
            );
        }
    }

    #[test]
    fn test_language_stats() {
        let stats = LanguageStats::calculate();
        assert!(stats.total_languages > 0, "should have languages");
        assert!(stats.families > 0, "should have families");
    }

    #[test]
    fn test_same_family() {
        // Elixir and Erlang are both BEAM
        assert!(
            same_family("elixir", "erlang"),
            "elixir and erlang are BEAM"
        );

        // Rust and Elixir are different families
        assert!(
            !same_family("rust", "elixir"),
            "rust and elixir are different families"
        );

        // Unknown languages
        assert!(
            !same_family("unknown1", "unknown2"),
            "unknown languages return false"
        );
    }

    #[test]
    fn test_supports_feature_ast_grep() {
        assert!(
            supports_feature("rust", LanguageCapability::ASTGrep),
            "rust supports ast-grep"
        );
    }

    #[test]
    fn test_supports_feature_tree_sitter() {
        assert!(
            supports_feature("rust", LanguageCapability::TreeSitter),
            "rust has tree-sitter"
        );
    }

    #[test]
    fn test_supports_feature_security() {
        assert!(
            supports_feature("rust", LanguageCapability::Security),
            "rust supports security analysis"
        );
        assert!(
            supports_feature("elixir", LanguageCapability::Security),
            "elixir supports security analysis"
        );
    }

    #[test]
    fn test_supports_feature_unknown_language() {
        assert!(
            !supports_feature("nonexistent", LanguageCapability::RCA),
            "unknown language returns false"
        );
    }

    #[test]
    fn test_supports_feature_complexity() {
        // Compiled languages support complexity
        assert!(
            supports_feature("rust", LanguageCapability::Complexity),
            "rust (compiled) supports complexity"
        );
        // Python is explicitly listed
        assert!(
            supports_feature("python", LanguageCapability::Complexity),
            "python supports complexity"
        );
    }

    #[test]
    fn test_supports_feature_performance() {
        assert!(
            supports_feature("rust", LanguageCapability::Performance),
            "rust supports performance analysis"
        );
    }
}
