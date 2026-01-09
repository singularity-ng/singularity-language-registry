use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Input Linguist YAML or JSON file (languages.yml)
    #[arg(long)]
    input: PathBuf,
    /// Output snapshot JSON file
    #[arg(long)]
    output: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub tree_sitter_language: Option<String>,
    #[serde(default)]
    pub rca_supported: bool,
    #[serde(default)]
    pub ast_grep_supported: bool,
    #[serde(default)]
    pub mime_types: Vec<String>,
    pub family: Option<String>,
    #[serde(default)]
    pub is_compiled: bool,
    #[serde(default = "String::new")]
    pub language_type: String,
    #[serde(default)]
    pub pattern_signatures: serde_json::Value,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = args.input;
    let output = args.output;

    // Read input; support JSON or YAML
    let contents = std::fs::read_to_string(&input)?;
    let map: serde_yaml::Value = if input.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&contents)?
    } else {
        serde_yaml::from_str(&contents)?
    };

    let mut snapshots: Vec<SnapshotEntry> = Vec::new();

    if let Some(obj) = map.as_mapping() {
        for (k, v) in obj {
            let display_name = k.as_str().unwrap_or_default().to_string();
            // Normalize ID to lowercase for consistent lookups
            // (Path::extension() returns "rs" not ".rs", and get_language expects lowercase)
            let id = display_name.to_lowercase();
            let name = display_name;
            // Map some fields
            // Strip leading dots from extensions since Path::extension() returns without dots
            let extensions: Vec<String> = v
                .get(&serde_yaml::Value::from("extensions"))
                .and_then(|x| x.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|e| {
                            e.as_str().map(|s| {
                                // Strip leading dot if present (Linguist uses ".rs", we need "rs")
                                s.strip_prefix('.').unwrap_or(s).to_string()
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            let aliases = v
                .get(&serde_yaml::Value::from("aliases"))
                .and_then(|x| x.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|e| e.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let mime_types = v
                .get(&serde_yaml::Value::from("mime_types"))
                .and_then(|x| x.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|e| e.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let tree_sitter_language = v
                .get(&serde_yaml::Value::from("tree_sitter_language"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());

            snapshots.push(SnapshotEntry {
                id,
                name,
                extensions,
                aliases,
                tree_sitter_language,
                rca_supported: false,
                ast_grep_supported: true,
                mime_types,
                family: None,
                is_compiled: false,
                language_type: "programming".to_string(),
                pattern_signatures: serde_json::Value::Null,
            });
        }
    }

    let f = File::create(output)?;
    to_writer_pretty(f, &snapshots)?;

    Ok(())
}
