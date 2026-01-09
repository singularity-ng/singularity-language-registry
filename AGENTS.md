# Agent Documentation - Singularity Language Registry (DEVELOPERS)

> **For LLMs and AI Agents:** This file is for DEVELOPERS working ON this library (contributing, modifying code). For USERS of the library, see the AGENTS.md in releases which has usage documentation.

> **Context:** You (the AI) are helping develop/maintain this library, not use it.

---

## Project Overview

**Name:** Singularity Language Registry
**Type:** Rust library crate
**Purpose:** Centralized language detection and metadata registry for code analysis tools
**License:** Proprietary (All Rights Reserved)
**Latest Version:** See GitHub releases

### What It Does

Provides production-ready language detection for **600+ programming languages** (from GitHub Linguist) with:
- File extension-based detection
- Content-based detection (shebang, patterns, heuristics)
- Language family grouping (BEAM, Systems, Web, JVM, etc.)
- File classification (vendored, generated, binary, documentation)
- Capability tracking (RCA, AST-Grep, linting, parsing, security, performance)
- Zero dependencies on other Singularity crates

### Key Features

- **600+ languages** - Complete GitHub Linguist integration
- **Fast lookups** - Optimized HashMaps for O(1) access
- **Content detection** - Shebang parsing, heuristics, pattern matching
- **File classification** - Skip vendored/generated files automatically
- **Comprehensive metadata** - Extensions, aliases, MIME types, families, linters
- **Zero warnings** - Strictest Clippy validation (pedantic + nursery)
- **Serializable** - All data structures support serde
- **Well-tested** - 89+ tests, high coverage

---

## Installation

### Option 1: Via Git Tag (Recommended for Proprietary)

```toml
[dependencies]
singularity-language-registry = { git = "https://github.com/Singularity-ng/singularity-language-registry", tag = "v0.1.0" }
```

### Option 2: From .crate File (Download from GitHub Release)

```bash
cargo install --path singularity-language-registry-0.1.0.crate
```

### Option 3: Local Path Dependency

```toml
[dependencies]
singularity-language-registry = { path = "./path/to/singularity-language-registry" }
```

### License Requirement

This is proprietary software. You must have a valid license agreement to use it.

---

## API Reference

### Core Detection Functions

#### `detect_language(path: &Path) -> Result<&LanguageInfo>`
Detects language from file path (extension-based).

```rust
use singularity_language_registry::detect_language;
use std::path::Path;

let path = Path::new("example.rs");
let lang = detect_language(path)?;
println!("Language: {}", lang.name);  // "Rust"
```

#### `detect_from_content(content: &str, fallback_ext: Option<&str>) -> Option<&LanguageInfo>`
Detects language from file content (shebang or patterns).

```rust
let python_code = "#!/usr/bin/env python3\nimport sys";
if let Some(lang) = detect_from_content(python_code, None) {
    println!("Detected: {}", lang.name);  // "Python"
}
```

#### `detect_from_shebang(content: &str) -> Option<&LanguageInfo>`
Extracts and detects from shebang line only.

```rust
let script = "#!/bin/bash\necho hello";
if let Some(lang) = detect_from_shebang(script) {
    println!("Detected: {}", lang.name);  // "Bash"
}
```

#### `detect_from_patterns(content: &str) -> Option<&LanguageInfo>`
Detects from content patterns (syntax signatures).

```rust
let elixir_code = "defmodule MyModule do\n  def hello, do: :world\nend";
if let Some(lang) = detect_from_patterns(elixir_code) {
    println!("Detected: {}", lang.name);  // "Elixir"
}
```

#### `detect_special_files(filename: &str) -> Option<&LanguageInfo>`
Detects special files (Makefile, Dockerfile, etc.).

```rust
if let Some(lang) = detect_special_files("Makefile") {
    println!("Detected: {}", lang.name);  // "Make"
}
```

#### `detection_confidence(path: &Path, content: &str) -> f32`
Returns confidence score (0.0-1.0) for detection accuracy.

```rust
let path = Path::new("example.rs");
let content = "fn main() {}";
let confidence = detection_confidence(path, content);
println!("Confidence: {:.2}%", confidence * 100.0);
```

---

### Registry Access Functions

#### `get_language(id: &str) -> Option<&LanguageInfo>`
Get language by ID.

```rust
if let Some(lang) = get_language("rust") {
    println!("Extensions: {:?}", lang.extensions);
}
```

#### `get_language_by_alias(alias: &str) -> Option<&LanguageInfo>`
Get language by alias (e.g., "js" → JavaScript).

```rust
if let Some(lang) = get_language_by_alias("js") {
    println!("Name: {}", lang.name);  // "JavaScript"
}
```

#### `get_language_by_mime_type(mime: &str) -> Option<&LanguageInfo>`
Get language by MIME type.

```rust
if let Some(lang) = get_language_by_mime_type("application/json") {
    println!("Name: {}", lang.name);  // "JSON"
}
```

#### `supported_languages() -> Vec<&LanguageInfo>`
Get all supported languages.

```rust
let all_langs = supported_languages();
println!("Total: {} languages", all_langs.len());
```

#### `rca_supported_languages() -> Vec<&LanguageInfo>`
Get languages with RCA support.

```rust
let rca_langs = rca_supported_languages();
for lang in rca_langs {
    println!("{} supports RCA", lang.name);
}
```

#### `ast_grep_supported_languages() -> Vec<&LanguageInfo>`
Get languages with AST-Grep support.

```rust
let ast_langs = ast_grep_supported_languages();
for lang in ast_langs {
    println!("{} supports AST-Grep", lang.name);
}
```

---

### Utility Functions

#### `languages_by_families() -> HashMap<LanguageFamily, Vec<&LanguageInfo>>`
Group languages by family.

```rust
let by_family = languages_by_families();
for (family, langs) in by_family {
    println!("{:?}: {} languages", family, langs.len());
}
```

#### `same_family(lang1: &str, lang2: &str) -> bool`
Check if two languages are in the same family.

```rust
if same_family("elixir", "erlang") {
    println!("Both are BEAM languages");
}
```

#### `recommended_linters(language: &str) -> Vec<String>`
Get recommended linters for a language.

```rust
let linters = recommended_linters("rust");
println!("Recommended: {:?}", linters);  // ["clippy", "rust-analyzer"]
```

#### `file_patterns(language: &str) -> Vec<String>`
Get common file patterns for a language.

```rust
let patterns = file_patterns("rust");
println!("Patterns: {:?}", patterns);  // ["*.rs", "Cargo.toml"]
```

#### `supports_feature(language: &str, capability: LanguageCapability) -> bool`
Check if language supports a capability.

```rust
use singularity_language_registry::LanguageCapability;

if supports_feature("rust", LanguageCapability::Security) {
    println!("Rust supports security analysis");
}

// Available capabilities: RCA, ASTGrep, Linting, Parsing, CodeEngine,
// TreeSitter, Complexity, Security, Performance
```

#### Capability registration

Downstream engines mark the subset of languages they actually implemented by toggling the `LanguageCapability` bits via `set_language_capability` or `register_capability_support`. The registry ships with all capabilities disabled, so every engine can take ownership of its own flag without racing.

```rust
use singularity_language_registry::{
    languages_with_capability, register_capability_support, set_language_capability,
    LanguageCapability,
};

set_language_capability("rust", LanguageCapability::Linting, true)?;

register_capability_support(
    LanguageCapability::Parsing,
    &["rust", "python", "javascript", "go"],
)?;

let linting = languages_with_capability(LanguageCapability::Linting);
println!("Linting-ready: {} languages", linting.len());
```

---

## Data Structures

### `LanguageInfo`

```rust
pub struct LanguageInfo {
    pub id: &'static str,              // "rust"
    pub name: &'static str,            // "Rust"
    pub extensions: Vec<String>,       // ["rs"]
    pub aliases: Vec<String>,          // ["rust-lang", "rustlang"]
    pub mime_types: Vec<String>,       // ["text/x-rust"]
    pub family: Option<String>,        // Some("Systems")
    pub is_compiled: bool,             // true
    pub language_type: String,         // "programming"
    pub tree_sitter_language: Option<String>, // Some("rust")
    pub linters: Vec<String>,          // ["clippy", "rustfmt"]
    pub file_patterns: Vec<String>,    // ["Cargo.toml", "Cargo.lock"]
    pub rca_supported: AtomicBool,     // runtime toggleable
    pub ast_grep_supported: AtomicBool, // runtime toggleable
    pub capabilities: AtomicU32,       // runtime bits for all capabilities
    pub pattern_signatures: PatternSignatures, // syntax patterns
}
```

### `LanguageFamily`

```rust
pub enum LanguageFamily {
    BEAM,       // Elixir, Erlang, Gleam
    Systems,    // Rust, C, C++, Go
    Web,        // JavaScript, TypeScript, Python
    Scripting,  // Bash, Lua
    Enterprise, // Java, Kotlin, C#
    Data,       // JSON, YAML, TOML, SQL
    Markup,     // Markdown
    Config,     // Dockerfile, Makefile
}
```

### `CompilationType`

```rust
pub enum CompilationType {
    Compiled,     // Rust, C, C++, Go
    Interpreted,  // Python, JavaScript
    Bytecode,     // Java, Elixir
    JIT,          // JavaScript (V8)
}
```

### `LanguageCapability`

```rust
pub enum LanguageCapability {
    RCA,         // Runtime Code Analysis
    ASTGrep,     // AST-based pattern matching
    Linting,     // Linting/formatting support
    Parsing,     // Tree-sitter parsing available
    CodeEngine,  // Code engine analysis
    TreeSitter,  // Tree-sitter grammar available
    Complexity,  // Cyclomatic complexity analysis
    Security,    // Security vulnerability scanning
    Performance, // Performance profiling
}
```

---

## Supported Languages

**600+ languages from GitHub Linguist**, including:

### BEAM Family
- Elixir (`.ex`, `.exs`) - with linters: credo, mix format
- Erlang (`.erl`, `.hrl`) - with linters: dialyzer
- Gleam (`.gleam`)
- LFE (`.lfe`)

### Systems Family
- Rust (`.rs`) - with linters: clippy, rustfmt
- C (`.c`, `.h`) - with linters: clang-tidy, cppcheck
- C++ (`.cpp`, `.hpp`, `.cc`, `.cxx`)
- Go (`.go`) - with linters: golangci-lint, gofmt
- Zig (`.zig`)
- Assembly (`.asm`, `.s`)

### Web Family
- JavaScript (`.js`, `.mjs`, `.cjs`) - with linters: eslint, prettier
- TypeScript (`.ts`, `.tsx`)
- Python (`.py`, `.pyw`) - with linters: pylint, black, mypy
- HTML, CSS, SCSS, Vue, Svelte, and more

### JVM Family
- Java (`.java`) - with linters: spotbugs, checkstyle
- Kotlin (`.kt`, `.kts`)
- Scala (`.scala`)
- Clojure (`.clj`, `.cljs`)
- Groovy (`.groovy`)

### Functional Languages
- Haskell (`.hs`)
- OCaml (`.ml`)
- F# (`.fs`)
- Elm (`.elm`)
- PureScript (`.purs`)

### Scripting Family
- Bash (`.sh`, `.bash`)
- Ruby (`.rb`) - with linters: rubocop
- Perl (`.pl`, `.pm`)
- PHP (`.php`)
- Lua (`.lua`)

### Data & Config
- JSON, YAML, TOML, XML
- SQL, GraphQL
- Dockerfile, Makefile, Nix, Terraform

**Use `supported_languages()` for the complete list.**

---

## Common Patterns

### Pattern 1: Simple File Detection

```rust
use singularity_language_registry::detect_language;
use std::path::Path;

fn detect_file(path: &Path) -> Result<String, String> {
    detect_language(path)
        .map(|lang| lang.name.to_string())
        .map_err(|e| format!("Detection failed: {}", e))
}
```

### Pattern 2: Content-Based with Fallback

```rust
use singularity_language_registry::{detect_from_content, detect_language};
use std::path::Path;

fn detect_with_fallback(path: &Path, content: &str) -> Option<&LanguageInfo> {
    // Try content-based first
    detect_from_content(content, path.extension()?.to_str())
        // Fall back to path-based
        .or_else(|| detect_language(path).ok())
}
```

### Pattern 3: Filter by Capability Support

```rust
use singularity_language_registry::{supported_languages, supports_feature, LanguageCapability};

fn security_supported() -> Vec<String> {
    supported_languages()
        .into_iter()
        .filter(|lang| supports_feature(lang.id, LanguageCapability::Security))
        .map(|lang| lang.name.to_string())
        .collect()
}
```

### Pattern 4: Language Family Analysis

```rust
use singularity_language_registry::{languages_by_families, LanguageFamily};

fn count_by_family() -> HashMap<String, usize> {
    languages_by_families()
        .into_iter()
        .map(|(family, langs)| (format!("{:?}", family), langs.len()))
        .collect()
}
```

---

## Quality Assurance

### Zero Warnings Tolerance
- ✅ Clippy pedantic + nursery lints
- ✅ `-D warnings` (all warnings are errors)
- ✅ See `clippy-report.md` in releases

### Security
- ✅ No known vulnerabilities (cargo-audit)
- ✅ All dependencies verified (cargo-deny)
- ✅ SBOM provided with all releases
- ✅ See `security-audit.md` in releases

### Testing
- ✅ All tests passing (release mode)
- ✅ High test coverage
- ✅ See `coverage-report.md` in releases

### Documentation
- ✅ All public APIs documented
- ✅ Examples included
- ✅ Builds on docs.rs

---

## Release Artifacts

Every GitHub release includes:

1. **Crate Package** - `singularity-language-registry-X.Y.Z.crate`
2. **Installation Guide** - `INSTALL.md`
3. **Package Contents** - `PACKAGE_CONTENTS.txt`
4. **Quality Reports Archive** - `release-reports-vX.Y.Z.tar.gz`
   - CHANGELOG.md
   - RELEASE_SUMMARY.md
   - clippy-report.md (zero warnings proof)
   - security-audit.md (vulnerability scan)
   - sbom.md (dependencies + licenses)
   - coverage-report.md (test coverage)
   - build-info.md (build environment)
   - dependency-report.md (outdated deps)

---

## Error Handling

The library uses explicit error handling:

```rust
// Returns Result for fallible operations
pub fn detect_language(path: &Path) -> Result<&LanguageInfo, DetectionError>

// Returns Option for nullable operations
pub fn detect_from_content(content: &str, fallback: Option<&str>) -> Option<&LanguageInfo>

// Custom error type
pub enum DetectionError {
    UnsupportedExtension(String),
    InvalidPath,
    NoLanguageFound,
}
```

**No panics** - All errors are explicit and handle-able.

---

## Performance Characteristics

- **Time Complexity:** O(1) for extension-based detection (HashMap lookup)
- **Space Complexity:** O(1) - All data is `&'static` (compile-time)
- **Allocation:** Zero allocations for most operations (static data)
- **Thread Safety:** 100% thread-safe (immutable static data)

---

## Design Principles

1. **No dependencies** - Zero runtime dependencies on other Singularity crates
2. **Static data** - All language definitions are compile-time constants
3. **Explicit behavior** - No fallbacks, no guessing, clear errors
4. **Fast lookups** - HashMap-based for O(1) access
5. **Serializable** - All types support serde for easy data exchange
6. **Well-tested** - Comprehensive test suite
7. **Zero warnings** - Strictest possible code quality standards

---

## Common Use Cases

### Use Case 1: Code Analysis Tool

```rust
// Detect language and run appropriate analysis
let path = Path::new("src/main.rs");
if let Ok(lang) = detect_language(path) {
    match lang.family {
        LanguageFamily::Systems => run_systems_analysis(path),
        LanguageFamily::Web => run_web_analysis(path),
        _ => run_generic_analysis(path),
    }
}
```

### Use Case 2: Build Tool

```rust
// Determine build command based on language
let lang = detect_language(path)?;
let build_cmd = match lang.id {
    "rust" => "cargo build",
    "go" => "go build",
    "javascript" => "npm run build",
    _ => return Err("Unsupported language"),
};
```

### Use Case 3: Linting Pipeline

```rust
// Run appropriate linters
if let Ok(lang) = detect_language(path) {
    let linters = recommended_linters(lang.id);
    for linter in linters {
        run_linter(linter, path)?;
    }
}
```

---

## Constraints and Limitations

1. **No content modification** - Read-only library, doesn't modify files
2. **No network** - Completely offline, no API calls
3. **No filesystem access** - Only analyzes paths/content you provide
4. **English-centric** - Language names and docs in English only
5. **Generated data** - Language definitions from GitHub Linguist (auto-generated)

---

## Version History

See `CHANGELOG.md` in releases for complete history.

---

## Support and Issues

- **Repository:** https://github.com/Singularity-ng/singularity-language-registry
- **Documentation:** This file + rustdoc + GitHub releases
- **Issues:** GitHub Issues (for licensed users only)
- **License:** Proprietary - contact Singularity Team for licensing

---

## For AI Agents: Code Generation Hints

When generating code using this library:

1. **Always handle errors** - Use `Result`/`Option` properly
2. **Check feature support** - Use `supports_feature()` before assuming capabilities
3. **Use type-safe enums** - Don't string match, use `LanguageFamily`, etc.
4. **Leverage static data** - No initialization needed, direct access
5. **Zero allocations** - Most operations return `&'static` refs
6. **Thread-safe** - Can use from multiple threads without locking

### Example: Complete Detection Flow

```rust
use singularity_language_registry::*;
use std::path::Path;

fn analyze_file(path: &Path, content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try content-based detection first
    let lang = detect_from_content(content, path.extension().and_then(|s| s.to_str()))
        // Fall back to path-based
        .or_else(|| detect_language(path).ok())
        .ok_or("Unable to detect language")?;

    // Check if RCA is supported
    if !lang.rca_supported {
        return Err(format!("{} does not support RCA", lang.name).into());
    }

    // Get confidence score
    let confidence = detection_confidence(path, content);
    if confidence < 0.8 {
        eprintln!("Warning: Low confidence detection ({:.2}%)", confidence * 100.0);
    }

    Ok(format!("Detected {} (family: {:?})", lang.name, lang.family))
}
```

---

**End of Agent Documentation**

This file is optimized for LLM understanding and should be included in all releases for AI agent consumption.
