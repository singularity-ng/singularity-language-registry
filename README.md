# Singularity Language Registry

[![Documentation](https://img.shields.io/badge/docs-rustdoc-blue.svg)](https://singularity-ng.github.io/singularity-language-registry/)
[![CI](https://github.com/singularity-ng/singularity-language-registry/workflows/CI/badge.svg)](https://github.com/singularity-ng/singularity-language-registry/actions)

Centralized language registry for all Singularity analysis engines.

**[📚 Documentation](https://singularity-ng.github.io/singularity-language-registry/)** | **[⚡ Installation Guide](./INSTALLATION.md)** | **[🔧 Setup Guide](./DOCS_SETUP.md)**

## Purpose

This crate provides a single source of truth for language definitions across:
- `singularity-parsing-engine` - AST extraction and parsing
- `singularity-analysis-engine` - Code metrics and complexity analysis
- `singularity-code-engine` - Code quality orchestration
- `singularity-linting-engine` - Quality gates and linting

## Design

**Pure language infrastructure** - no dependencies on other Singularity crates.

All engines depend on this registry, not the other way around:

```
language_registry (independent)
    ↑ ↑ ↑ ↑
    │ │ │ └─ parsing_engine
    │ │ └─── analysis_engine
    │ └───── code_engine
    └─────── linting_engine
```

## Features

- **600+ languages** from GitHub Linguist with complete metadata
- **Fast lookups** - optimized HashMaps for extensions, aliases, MIME types
- **Content detection** - shebang parsing, heuristics, and pattern matching
- **File classification** - vendored, generated, binary, documentation detection
- **Family grouping** - BEAM, Systems, Web, JVM, C-like, Scripting, and more
- **Capability tracking** - RCA, AST-Grep, linting, parsing, security, performance
- **Zero dependencies** on other Singularity crates
- **Serializable** - all language info can be exported as JSON

## Installation

> **📖 See [INSTALLATION.md](./INSTALLATION.md) for complete guide including Mix (Elixir) dependencies and binary downloads**

### Quick Start - Rust

```bash
cargo add singularity-language-registry
```

Or in `Cargo.toml`:
```toml
[dependencies]
singularity-language-registry = { git = "https://github.com/Singularity-ng/singularity-language-registry", tag = "latest" }
```

### Using Nix

```bash
# Add to your flake.nix inputs
inputs.singularity-language-registry.url = "github:Singularity-ng/singularity-language-registry";

# Or use directly with Nix
nix build github:Singularity-ng/singularity-language-registry

# Enter development shell
nix develop github:Singularity-ng/singularity-language-registry
```

### With FlakeHub (Fastest - Binary Cache)

```bash
# FlakeHub provides pre-built binaries for faster installation
nix build "https://flakehub.com/f/Singularity-ng/singularity-language-registry/*.tar.gz"

# Or add to your flake.nix
inputs.singularity-language-registry.url = "https://flakehub.com/f/Singularity-ng/singularity-language-registry/0.1.*.tar.gz";
```

## Usage

### Basic Language Detection

```rust
use singularity_language_registry::{detect_language, get_language};
use std::path::Path;

// Detect from file path
let path = Path::new("example.rs");
if let Ok(lang) = detect_language(path) {
    println!("Language: {}", lang.name);
    println!("Family: {:?}", lang.family);
}

// Get language by ID
if let Some(lang) = get_language("elixir") {
    println!("Extensions: {:?}", lang.extensions);
    println!("RCA supported: {}", lang.rca_supported);
}
```

### Content-Based Detection

```rust
use singularity_language_registry::{detect_from_content, detect_from_shebang};

let python_code = "#!/usr/bin/env python3\nimport sys";
if let Some(lang) = detect_from_content(python_code, None) {
    println!("Detected: {}", lang.name);
}
```

### Language Statistics & Utilities

```rust
use singularity_language_registry::{
    LanguageStats,
    LanguageCapability,
    languages_by_families,
    same_family,
    recommended_linters,
    supports_feature,
};

// Get statistics
let stats = LanguageStats::calculate();
println!("Total languages: {}", stats.total_languages);

// Check family relationships
if same_family("elixir", "erlang") {
    println!("Both are BEAM languages!");
}

// Get recommended tools (from metadata or fallback)
let linters = recommended_linters("rust");
println!("Rust linters: {:?}", linters);

// Check capability support
if supports_feature("rust", LanguageCapability::Security) {
    println!("Rust supports security analysis!");
}
```

## API Documentation

### Core Types

- `LanguageInfo` - Complete language metadata (name, extensions, family, linters, etc.)
- `LanguageRegistry` - Main registry struct (usually accessed via LANGUAGE_REGISTRY)
- `LanguageCapability` - Enum of analysis capabilities (RCA, ASTGrep, Security, etc.)
- `PatternSignatures` - Language syntax patterns for detection
- `LanguageStats` - Statistics about language support
- `FileClassifier` - Classify files as vendored, generated, binary, or documentation

### Detection Functions

- `detect_language(path)` - Detect from file path
- `detect_from_content(content, fallback_ext)` - Detect from file content
- `detect_from_shebang(content)` - Detect from shebang line
- `detect_from_patterns(content)` - Detect from content patterns
- `detect_from_heuristics(ext, content)` - Disambiguate by extension + content heuristics
- `detect_special_files(filename)` - Detect special files (Makefile, Dockerfile, etc.)
- `is_detectable(language)` - Check if a language is detectable

### Registry Functions

- `get_language(id)` - Get language by ID
- `get_language_by_alias(alias)` - Get language by alias (e.g., "js")
- `get_language_by_mime_type(mime)` - Get language by MIME type
- `supported_languages()` - Get all supported languages
- `rca_supported_languages()` - Get languages with RCA support
- `ast_grep_supported_languages()` - Get languages with AST-Grep support

### Utility Functions

- `languages_by_families()` - Group languages by family
- `same_family(lang1, lang2)` - Check if two languages are in same family
- `recommended_linters(language)` - Get recommended linters for a language
- `file_patterns(language)` - Get common file patterns for a language
- `supports_feature(language, feature)` - Check if language supports a feature

### Capability Registration

Every language entry starts with a clean slate for runtime capabilities (RCA, linting, parsing, etc.). Downstream engines flip bits as they ship support via `LanguageCapability` and the helper APIs.

```rust
use singularity_language_registry::{
    LanguageCapability, languages_with_capability, register_capability_support,
    set_language_capability,
};

// Enable linting support for Rust at runtime.
set_language_capability("rust", LanguageCapability::Linting, true)?;

// Enumerate the languages that now advertise linting.
let linting_langs = languages_with_capability(LanguageCapability::Linting);
println!("Linting-ready languages: {}", linting_langs.len());

// Bulk-enable parsing for a suite of languages.
register_capability_support(
    LanguageCapability::Parsing,
    &["rust", "go", "python", "javascript"],
)?;
```

`register_rca_capabilities` now builds on the same mechanism (`LanguageCapability::RCA`), so all capability registration flows re-use a single atomic bitfield per language.

## Supported Languages

600+ languages from GitHub Linguist, including:

- **BEAM**: Elixir, Erlang, Gleam, LFE
- **Systems**: Rust, C, C++, Go, Zig, Assembly
- **Web**: JavaScript, TypeScript, HTML, CSS, SCSS, Vue, Svelte
- **JVM**: Java, Kotlin, Scala, Clojure, Groovy
- **Scripting**: Python, Ruby, Perl, PHP, Lua, Shell
- **Functional**: Haskell, OCaml, F#, Elm, PureScript
- **Data**: SQL, GraphQL, JSON, YAML, TOML, XML
- **Config**: Dockerfile, Makefile, Nix, Terraform, Ansible
- **And many more...** (see `supported_languages()` for full list)

## File Classification

Classify files using GitHub Linguist patterns:

```rust
use singularity_language_registry::{FileClassifier, FileClass};

let classifier = FileClassifier::default();

// Check if file should be analyzed
if classifier.should_analyze("src/main.rs") {
    println!("Analyze this file");
}

// Get file classification
match classifier.classify("node_modules/pkg/index.js") {
    FileClass::Vendored => println!("Skip vendored file"),
    FileClass::Generated => println!("Skip generated file"),
    FileClass::Source => println!("Analyze source file"),
    _ => {}
}
```

## Zero Dependencies

This crate has no dependencies on other Singularity crates, making it safe for all engines to import without circular dependencies.

## Development

### Documentation

Generate and view documentation locally:

```bash
# Using Nix (recommended)
nix run .#docs

# Using cargo
cargo doc --all-features --open

# Build documentation package with Nix
nix build .#docs
```

The documentation is automatically published to GitHub Pages when changes are pushed to `main`. See [DOCS_SETUP.md](./DOCS_SETUP.md) for complete documentation setup and deployment instructions.

### Testing

```bash
# Run all tests
cargo test --all-features

# Run with Nix
nix flake check
```

### Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines and contribution process.

## License

**Proprietary Software - All Rights Reserved**

Copyright (c) 2025 Singularity Team. All rights reserved.

This software is proprietary and confidential. Unauthorized copying, distribution, modification, or use of this software, via any medium, is strictly prohibited without explicit written permission from Singularity Team.

See [LICENSE](./LICENSE) for complete terms and conditions.

For licensing inquiries, please contact the Singularity Team.
