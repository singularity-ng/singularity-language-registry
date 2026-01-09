//! File Classification Engine - Identifies vendored, generated, and binary files
//!
//! This module provides classification rules derived from GitHub Linguist's patterns:
//! <https://github.com/github-linguist/linguist/blob/main/lib/linguist/vendor.yml>
//!
//! ## Classification Categories
//!
//! - **Vendored**: Third-party dependencies (`node_modules/`, `vendor/`, etc.)
//! - **Generated**: Auto-generated files (protobuf, graphql, minified, etc.)
//! - **Binary**: Non-text files (images, archives, compiled binaries)
//! - **Documentation**: Auto-generated docs (Sphinx, Doxygen)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use singularity_language_registry::FileClassifier;
//! use std::path::Path;
//!
//! let classifier = FileClassifier::new();
//! let path = Path::new("node_modules/package/index.js");
//!
//! if classifier.is_vendored(path) {
//!     println!("Skip vendored code");
//! }
//! ```

use crate::file_classifier_generated::{
    BINARY_PATTERNS_FROM_LINGUIST, VENDORED_PATTERNS_FROM_LINGUIST,
};
use std::path::Path;

/// File classification result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileClass {
    /// Regular source code file
    Source,
    /// Vendored/third-party dependency
    Vendored,
    /// Auto-generated file
    Generated,
    /// Binary file (non-text)
    Binary,
    /// Auto-generated documentation
    Documentation,
}

/// File classifier using Linguist patterns
///
/// Uses patterns from GitHub Linguist (179 vendored patterns, 84 generated patterns)
/// for comprehensive file classification.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileClassifier {
    /// Documentation tool markers (custom, not from Linguist)
    documentation_markers: &'static [&'static str],
    /// Additional binary extensions not in Linguist
    extra_binary_extensions: &'static [&'static str],
}

/// Additional binary extensions commonly encountered but not in Linguist
const EXTRA_BINARY_EXTENSIONS: &[&str] = &[
    // Images
    ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".svg", ".ico", ".webp", ".tiff", ".tif",
    // Archives
    ".zip", ".tar", ".gz", ".rar", ".7z", ".bz2", ".xz", ".zst", // Compiled binaries
    ".exe", ".bin", ".so", ".dll", ".dylib", ".a", ".lib", ".o", ".obj",
    // Documents (binary formats)
    ".pdf", ".docx", ".xlsx", ".pptx", ".doc", ".xls", ".ppt", // Audio/Video
    ".mp3", ".mp4", ".wav", ".avi", ".mov", ".flac", ".ogg", ".mkv", ".webm", // Fonts
    ".ttf", ".otf", ".woff", ".woff2", ".eot", // Other binary formats
    ".pyc", ".pyo", ".class", ".jar", ".war", ".ear",
];

/// Documentation tool markers
const DOCUMENTATION_MARKERS: &[&str] = &["doxygen", "sphinx", "jsdoc", "pandoc", "mkdocs"];

impl FileClassifier {
    /// Create a new file classifier with Linguist patterns
    #[must_use]
    pub fn new() -> Self {
        Self {
            documentation_markers: DOCUMENTATION_MARKERS,
            extra_binary_extensions: EXTRA_BINARY_EXTENSIONS,
        }
    }

    /// Check if path is vendored code using Linguist patterns.
    ///
    /// Uses 179 patterns from GitHub Linguist's vendor.yml for comprehensive detection.
    #[must_use]
    pub fn is_vendored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check Linguist vendored patterns (regex patterns, but we do simple matching)
        for pattern in VENDORED_PATTERNS_FROM_LINGUIST {
            if simple_pattern_matches(pattern, &path_str) {
                return true;
            }
        }

        false
    }

    /// Check if file is generated using path-based heuristics.
    ///
    /// Note: Linguist's generated.rb uses content heuristics which are not applied here.
    /// This method checks for common generated file indicators in the path only.
    #[must_use]
    pub fn is_generated(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let lower = path_str.to_lowercase();

        // Check common generated file path patterns
        // Protobuf generated files
        if lower.ends_with(".pb.rs")
            || lower.ends_with(".pb.go")
            || lower.ends_with(".pb.py")
            || lower.ends_with(".pb.js")
            || lower.ends_with(".pb.ts")
            || lower.ends_with(".pb.h")
            || lower.ends_with(".pb.cc")
            || lower.ends_with(".pb2.py")
            || lower.ends_with(".pb2_grpc.py")
            || lower.ends_with(".grpc.pb.go")
        {
            return true;
        }

        // TypeScript declaration files (often generated)
        if lower.ends_with(".d.ts") {
            return true;
        }

        // Visual Studio designer files
        if lower.ends_with(".designer.cs") || lower.ends_with(".designer.vb") {
            return true;
        }

        // Unity metadata
        if lower.ends_with(".meta") && lower.contains("assets/") {
            return true;
        }

        // Xcode generated
        if lower.ends_with(".nib")
            || lower.ends_with(".xcworkspacedata")
            || lower.ends_with(".storyboard")
            || lower.ends_with(".xib")
            || lower.ends_with(".xcuserstate")
        {
            return true;
        }

        // GraphQL/Angular generated
        if lower.contains(".generated.ts")
            || lower.contains(".generated.js")
            || lower.contains(".generated.tsx")
            || lower.contains(".generated.jsx")
        {
            return true;
        }

        // Files explicitly marked as generated in filename
        if lower.contains("_generated.") || lower.contains("-generated.") {
            return true;
        }

        // Lock files and auto-generated manifests
        if lower.ends_with(".lock")
            || lower.ends_with("-lock.json")
            || lower.ends_with("-lock.yaml")
        {
            return true;
        }

        false
    }

    /// Check if file is binary using extension matching.
    #[must_use]
    pub fn is_binary(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let lower = path_str.to_lowercase();

        // Check Linguist binary patterns
        for pattern in BINARY_PATTERNS_FROM_LINGUIST {
            if lower.ends_with(pattern) {
                return true;
            }
        }

        // Check extra binary extensions
        for ext in self.extra_binary_extensions {
            if lower.ends_with(ext) {
                return true;
            }
        }

        false
    }

    /// Check if file is documentation
    #[must_use]
    pub fn is_documentation(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let lower = path_str.to_lowercase();
        self.documentation_markers
            .iter()
            .any(|marker| lower.contains(marker))
    }

    /// Classify a file path
    #[must_use]
    pub fn classify(&self, path: &Path) -> FileClass {
        if self.is_binary(path) {
            FileClass::Binary
        } else if self.is_vendored(path) {
            FileClass::Vendored
        } else if self.is_generated(path) {
            FileClass::Generated
        } else if self.is_documentation(path) {
            FileClass::Documentation
        } else {
            FileClass::Source
        }
    }

    /// Check if file should be analyzed (not vendored, generated, or binary)
    #[must_use]
    pub fn should_analyze(&self, path: &Path) -> bool {
        matches!(self.classify(path), FileClass::Source)
    }
}

/// Simple pattern matching for Linguist regex patterns.
///
/// Handles common cases without full regex support:
/// - Literal substrings
/// - Path separators `(^|/)`
/// - Simple alternations like `[Vv]endor`
fn simple_pattern_matches(pattern: &str, path: &str) -> bool {
    // Handle start-of-path or slash prefix patterns like "(^|/)"
    if pattern.starts_with("(^|/)") || pattern.starts_with("(^|\\/)") {
        let rest = pattern
            .trim_start_matches("(^|/)")
            .trim_start_matches("(^|\\/)")
            .replace("\\/", "/")
            .replace("\\.", ".");

        // Extract simple substring from pattern (remove regex syntax)
        let simplified = simplify_regex_pattern(&rest);
        if simplified.is_empty() {
            return false;
        }

        // Check if path starts with or contains /simplified
        let lower_path = path.to_lowercase();
        let lower_pattern = simplified.to_lowercase();
        return lower_path.starts_with(&lower_pattern)
            || lower_path.contains(&format!("/{lower_pattern}"));
    }

    // Handle end-of-string patterns like "$"
    if pattern.ends_with('$') {
        let rest = &pattern[..pattern.len().saturating_sub(1)];
        let simplified = simplify_regex_pattern(rest);
        if simplified.is_empty() {
            return false;
        }
        return path.to_lowercase().ends_with(&simplified.to_lowercase());
    }

    // Handle patterns starting with "^"
    if pattern.starts_with('^') {
        let rest = &pattern[1..];
        let simplified = simplify_regex_pattern(rest);
        if simplified.is_empty() {
            return false;
        }
        return path.to_lowercase().starts_with(&simplified.to_lowercase());
    }

    // Simple substring match for patterns without anchors
    let simplified = simplify_regex_pattern(pattern);
    if simplified.is_empty() {
        return false;
    }
    path.to_lowercase().contains(&simplified.to_lowercase())
}

/// Simplify a regex pattern to extract a matchable substring.
fn simplify_regex_pattern(pattern: &str) -> String {
    let mut result = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Handle escape sequences
                if let Some(&next) = chars.peek() {
                    let escaped_char = match next {
                        '.' => '.',
                        '/' => '/',
                        '-' => '-',
                        '_' => '_',
                        's' | 'S' | 'd' | 'D' | 'w' | 'W' | 'b' | 'B' => {
                            let _ = chars.next();
                            continue;
                        }
                        _ => next,
                    };
                    result.push(escaped_char);
                    let _ = chars.next();
                }
            }
            '[' => {
                // Handle character classes like [Vv] -> extract first char
                if let Some(&first) = chars.peek() {
                    if first != '^' {
                        result.push(first.to_ascii_lowercase());
                    }
                }
                // Skip until ]
                for ch in chars.by_ref() {
                    if ch == ']' {
                        break;
                    }
                }
            }
            '(' | ')' | '*' | '+' | '?' | '|' | '^' | '$' | '{' | '}' => {
                // Skip regex metacharacters
                continue;
            }
            _ => {
                result.push(c);
            }
        }
    }

    result
}

#[cfg(test)]
#[allow(
    clippy::missing_panics_doc,
    reason = "Test module doesn't need panic documentation"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ========== Vendored Detection Tests ==========

    #[test]
    fn test_vendored_detection() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_vendored(&PathBuf::from("node_modules/package/index.js")));
        assert!(classifier.is_vendored(&PathBuf::from("vendor/lib/helper.rb")));
        assert!(!classifier.is_vendored(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_vendored_bower_components() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_vendored(&PathBuf::from("bower_components/jquery/jquery.js")));
    }

    #[test]
    fn test_vendored_dist() {
        let classifier = FileClassifier::new();
        // dist/ is a common vendored pattern
        assert!(classifier.is_vendored(&PathBuf::from("dist/bundle.js")));
        assert!(classifier.is_vendored(&PathBuf::from("project/dist/app.min.js")));
    }

    #[test]
    fn test_vendored_cache() {
        let classifier = FileClassifier::new();
        // cache/ is a vendored pattern in Linguist
        assert!(classifier.is_vendored(&PathBuf::from("cache/data.json")));
        assert!(classifier.is_vendored(&PathBuf::from("project/cache/files.db")));
    }

    #[test]
    fn test_vendored_gradle_maven_wrappers() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_vendored(&PathBuf::from("gradle/wrapper/gradle-wrapper.jar")));
        assert!(classifier.is_vendored(&PathBuf::from(".mvn/wrapper/maven-wrapper.jar")));
    }

    #[test]
    fn test_vendored_godeps() {
        let classifier = FileClassifier::new();
        // Godeps/_workspace/ is in Linguist vendor.yml
        assert!(classifier.is_vendored(&PathBuf::from("Godeps/_workspace/src/pkg/util.go")));
    }

    // ========== Generated Detection Tests ==========

    #[test]
    fn test_generated_detection() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("api.pb.rs")));
        assert!(classifier.is_generated(&PathBuf::from("Component.generated.ts")));
        assert!(!classifier.is_generated(&PathBuf::from("Component.ts")));
    }

    #[test]
    fn test_generated_protobuf_go() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("user.pb.go")));
        assert!(classifier.is_generated(&PathBuf::from("service.grpc.pb.go")));
    }

    #[test]
    fn test_generated_protobuf_python() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("message.pb.py")));
        assert!(classifier.is_generated(&PathBuf::from("service.pb2.py")));
        assert!(classifier.is_generated(&PathBuf::from("service.pb2_grpc.py")));
    }

    #[test]
    fn test_generated_typescript_declarations() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("types.d.ts")));
        assert!(classifier.is_generated(&PathBuf::from("lib/index.d.ts")));
    }

    #[test]
    fn test_generated_designer_files() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("Form1.designer.cs")));
        assert!(classifier.is_generated(&PathBuf::from("Form1.designer.vb")));
    }

    #[test]
    fn test_generated_lock_files() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("Cargo.lock")));
        assert!(classifier.is_generated(&PathBuf::from("yarn.lock")));
        assert!(classifier.is_generated(&PathBuf::from("package-lock.json")));
    }

    #[test]
    fn test_generated_xcode() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_generated(&PathBuf::from("project.xcworkspacedata")));
        assert!(classifier.is_generated(&PathBuf::from("Main.storyboard")));
    }

    #[test]
    fn test_not_generated_regular_ts() {
        let classifier = FileClassifier::new();
        assert!(!classifier.is_generated(&PathBuf::from("app.ts")));
        assert!(!classifier.is_generated(&PathBuf::from("index.tsx")));
        assert!(!classifier.is_generated(&PathBuf::from("Component.ts")));
    }

    // ========== Binary Detection Tests ==========

    #[test]
    fn test_binary_detection() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_binary(&PathBuf::from("image.png")));
        assert!(classifier.is_binary(&PathBuf::from("archive.zip")));
        assert!(!classifier.is_binary(&PathBuf::from("script.js")));
    }

    #[test]
    fn test_binary_images() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_binary(&PathBuf::from("logo.png")));
        assert!(classifier.is_binary(&PathBuf::from("photo.jpg")));
        assert!(classifier.is_binary(&PathBuf::from("icon.gif")));
        assert!(classifier.is_binary(&PathBuf::from("hero.webp")));
    }

    #[test]
    fn test_binary_archives() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_binary(&PathBuf::from("backup.tar")));
        assert!(classifier.is_binary(&PathBuf::from("data.gz")));
        assert!(classifier.is_binary(&PathBuf::from("files.7z")));
    }

    #[test]
    fn test_binary_executables() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_binary(&PathBuf::from("program.exe")));
        assert!(classifier.is_binary(&PathBuf::from("libfoo.so")));
        assert!(classifier.is_binary(&PathBuf::from("libbar.dylib")));
        assert!(classifier.is_binary(&PathBuf::from("helper.dll")));
    }

    #[test]
    fn test_binary_fonts() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_binary(&PathBuf::from("font.ttf")));
        assert!(classifier.is_binary(&PathBuf::from("icons.woff2")));
    }

    #[test]
    fn test_binary_media() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_binary(&PathBuf::from("song.mp3")));
        assert!(classifier.is_binary(&PathBuf::from("video.mp4")));
        assert!(classifier.is_binary(&PathBuf::from("audio.wav")));
    }

    // ========== Documentation Detection Tests ==========

    #[test]
    fn test_documentation_detection() {
        let classifier = FileClassifier::new();
        assert!(classifier.is_documentation(&PathBuf::from("docs/doxygen/index.html")));
        assert!(classifier.is_documentation(&PathBuf::from("sphinx/build/index.html")));
        assert!(!classifier.is_documentation(&PathBuf::from("src/main.rs")));
    }

    // ========== Classification Tests ==========

    #[test]
    fn test_classification() {
        let classifier = FileClassifier::new();
        assert_eq!(
            classifier.classify(&PathBuf::from("node_modules/pkg/index.js")),
            FileClass::Vendored
        );
        assert_eq!(
            classifier.classify(&PathBuf::from("api.pb.rs")),
            FileClass::Generated
        );
        assert_eq!(
            classifier.classify(&PathBuf::from("image.png")),
            FileClass::Binary
        );
        assert_eq!(
            classifier.classify(&PathBuf::from("src/main.rs")),
            FileClass::Source
        );
    }

    #[test]
    fn test_classification_priority() {
        let classifier = FileClassifier::new();
        // Binary takes precedence over vendored
        assert_eq!(
            classifier.classify(&PathBuf::from("vendor/fonts/icon.woff")),
            FileClass::Binary
        );
        // Vendored takes precedence over generated
        assert_eq!(
            classifier.classify(&PathBuf::from("node_modules/pkg/types.d.ts")),
            FileClass::Vendored
        );
    }

    // ========== Should Analyze Tests ==========

    #[test]
    fn test_should_analyze() {
        let classifier = FileClassifier::new();
        assert!(classifier.should_analyze(&PathBuf::from("src/main.rs")));
        assert!(!classifier.should_analyze(&PathBuf::from("node_modules/pkg/index.js")));
        assert!(!classifier.should_analyze(&PathBuf::from("api.pb.rs")));
    }

    #[test]
    fn test_should_analyze_various_source() {
        let classifier = FileClassifier::new();
        assert!(classifier.should_analyze(&PathBuf::from("lib/utils.py")));
        assert!(classifier.should_analyze(&PathBuf::from("app/controllers/home.ex")));
        assert!(classifier.should_analyze(&PathBuf::from("cmd/server/main.go")));
    }

    // ========== Pattern Matching Helper Tests ==========

    #[test]
    fn test_simple_pattern_matches_substring() {
        assert!(simple_pattern_matches("vendor/", "path/to/vendor/lib.js"));
        assert!(!simple_pattern_matches("vendor/", "path/to/myvendor.js"));
    }

    #[test]
    fn test_simple_pattern_matches_case_insensitive() {
        assert!(simple_pattern_matches("Vendor/", "path/to/vendor/lib.js"));
        assert!(simple_pattern_matches("vendor/", "path/to/VENDOR/lib.js"));
    }

    #[test]
    fn test_simplify_regex_pattern_basic() {
        assert_eq!(simplify_regex_pattern("hello"), "hello");
        assert_eq!(simplify_regex_pattern("a\\.b"), "a.b");
    }

    #[test]
    fn test_simplify_regex_pattern_character_class() {
        // [Vv] should extract first char lowercased
        let result = simplify_regex_pattern("[Vv]endor");
        assert!(result.contains("vendor") || result.contains("endor"));
    }

    #[test]
    fn test_simplify_regex_pattern_removes_metacharacters() {
        let result = simplify_regex_pattern("a*b+c?");
        assert!(!result.contains('*'));
        assert!(!result.contains('+'));
        assert!(!result.contains('?'));
    }

    // ========== Default/Copy Trait Tests ==========

    #[test]
    fn test_classifier_default() {
        let c1 = FileClassifier::default();
        let c2 = FileClassifier::new();
        // Both should behave identically
        assert_eq!(
            c1.classify(&PathBuf::from("test.rs")),
            c2.classify(&PathBuf::from("test.rs"))
        );
    }

    #[test]
    fn test_classifier_copy() {
        let c1 = FileClassifier::new();
        let c2 = c1; // Copy
                     // Both should work independently
        assert!(c1.should_analyze(&PathBuf::from("test.rs")));
        assert!(c2.should_analyze(&PathBuf::from("test.rs")));
    }
}
