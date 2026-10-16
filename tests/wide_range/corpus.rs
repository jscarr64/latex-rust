//! Corpus: every LaTeX string in the repo (golds, docs, README, symbol
//! catalog, fuzz seeds) and a snapshot of Accumath's gold and doc strings.
//!
//! The Accumath snapshot is `tests/data/accumath_latex_corpus.txt`: records
//! `origin\nlatex\n\x1e\n`, taken read-only from the Accumath checkout's
//! `golds/**` (every `key: value` line whose value holds a control word) and
//! `doc/**/*.md` (math spans and backtick spans). It is a snapshot so the
//! test needs no sibling checkout.

use super::extract::{math_spans, toml_values};
use std::path::{Path, PathBuf};

/// One corpus string and where it came from.
pub struct Item {
    /// File and key, for failure messages.
    pub origin: String,
    /// The LaTeX string.
    pub latex: String,
    /// `Some(true)` when a gold says it must parse, `Some(false)` when a gold
    /// says it must be rejected, `None` when nothing is promised.
    pub must_parse: Option<bool>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn files_in(dir: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in rd {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

/// Strings from `golds/*.toml`: `input`, `pair`, `lhs`, `rhs`; a gold of kind
/// `err` must be rejected, a render or AST gold must parse, tokenizer golds promise nothing.
fn gold_items(out: &mut Vec<Item>) -> Result<(), String> {
    for path in files_in(&root().join("golds"), "toml")? {
        let text = read(&path)?;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        for (k, record) in text.split("[[").enumerate() {
            let kind = toml_values(record, &["kind"])
                .into_iter()
                .next()
                .unwrap_or_default();
            let promise = match kind.as_str() {
                "err" if name == "parse.toml" => Some(false),
                "err" | "err_parse" => None,
                "tokenize" | "tokenize_err" | "err_layout" | "" => None,
                _ => Some(true),
            };
            for s in toml_values(record, &["input"]) {
                out.push(Item {
                    origin: format!("golds/{name}#{k}:input"),
                    latex: s,
                    must_parse: promise,
                });
            }
            for s in toml_values(record, &["pair", "lhs", "rhs"]) {
                out.push(Item {
                    origin: format!("golds/{name}#{k}:pair"),
                    latex: s,
                    must_parse: None,
                });
            }
        }
    }
    Ok(())
}

/// Markdown spans from the README and `documents/*.md`.
fn doc_items(out: &mut Vec<Item>) -> Result<(), String> {
    let mut paths = vec![root().join("README.md"), root().join("CHANGELOG.md")];
    paths.extend(files_in(&root().join("documents"), "md")?);
    for path in paths {
        let text = read(&path)?;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        for (k, s) in math_spans(&text).into_iter().enumerate() {
            out.push(Item {
                origin: format!("{name}:span{k}"),
                latex: s,
                must_parse: None,
            });
        }
    }
    Ok(())
}

/// The `latex` column of `data/symbols.tsv`.
fn symbol_items(out: &mut Vec<Item>) -> Result<(), String> {
    let text = read(&root().join("data/symbols.tsv"))?;
    for (k, line) in text.lines().enumerate() {
        if line.starts_with('#') || k == 0 {
            continue;
        }
        if let Some(cell) = line.split('\t').nth(3) {
            out.push(Item {
                origin: format!("symbols.tsv:{k}"),
                latex: String::from(cell),
                must_parse: None,
            });
        }
    }
    Ok(())
}

/// Every file in `fuzz/corpus/parse` that is UTF-8.
fn seed_items(out: &mut Vec<Item>) -> Result<(), String> {
    let dir = root().join("fuzz/corpus/parse");
    let rd = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in rd {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        paths.push(entry.path());
    }
    paths.sort();
    for p in paths {
        if let Ok(s) = std::fs::read_to_string(&p) {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            out.push(Item {
                origin: format!("fuzz/corpus/parse/{name}"),
                latex: s,
                must_parse: None,
            });
        }
    }
    Ok(())
}

/// All repo strings.
pub fn repo_items() -> Result<Vec<Item>, String> {
    let mut out = Vec::new();
    gold_items(&mut out)?;
    doc_items(&mut out)?;
    symbol_items(&mut out)?;
    seed_items(&mut out)?;
    Ok(out)
}

/// The Accumath snapshot.
pub fn accumath_items() -> Result<Vec<Item>, String> {
    let text = read(&root().join("tests/data/accumath_latex_corpus.txt"))?;
    let mut out = Vec::new();
    for rec in text.split("\u{1e}\n") {
        let Some((origin, body)) = rec.split_once('\n') else {
            continue;
        };
        let body = body.strip_suffix('\n').unwrap_or(body);
        if body.is_empty() {
            continue;
        }
        out.push(Item {
            origin: String::from(origin),
            latex: String::from(body),
            must_parse: None,
        });
    }
    Ok(out)
}
