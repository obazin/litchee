//! Source guard: every dynamic string path segment must be percent-encoded.
//!
//! Caller-supplied path segments are wrapped with `http::segment(...)` and
//! interpolated as positional `{}` placeholders; the only inline *named*
//! interpolations (`{name}`) left in path literals are numeric ids that need no
//! encoding. This test re-reads the `src/api` source and fails if a path literal
//! introduces a named interpolation that is not a known numeric parameter — the
//! signature of a forgotten `http::segment` wrap that could reopen the
//! path-injection class fixed earlier. It catches the omission at CI time
//! without requiring the request layer to be re-architected.

use std::fs;
use std::path::{Path, PathBuf};

/// Numeric path parameters that are safe to interpolate without encoding.
const NUMERIC_PATH_PARAMS: &[&str] = &["player_id", "days", "seconds", "nb"];

#[test]
fn dynamic_path_segments_are_encoded() {
    let api_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api");
    let mut files = Vec::new();
    collect_rs_files(&api_dir, &mut files);
    assert!(
        !files.is_empty(),
        "no source files under {}",
        api_dir.display()
    );

    let mut violations = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("source file is readable");
        scan_file(&text, file, &api_dir, &mut violations);
    }
    assert!(violations.is_empty(), "{}", report(&violations));
}

/// Collects every `.rs` file under `dir`, recursively.
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("api dir is readable") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Records every non-allowlisted named path interpolation in `text`.
fn scan_file(text: &str, file: &Path, api_dir: &Path, out: &mut Vec<String>) {
    let rel = file.strip_prefix(api_dir).unwrap_or(file);
    for (idx, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue; // skip comments / doc lines (they hold API-doc path templates)
        }
        for name in named_path_interpolations(line) {
            if !NUMERIC_PATH_PARAMS.contains(&name.as_str()) {
                out.push(format!("{}:{}: {{{name}}}", rel.display(), idx + 1));
            }
        }
    }
}

/// Names of `{name}` interpolations inside path string literals (those starting
/// with `/`) on one line. Positional `{}` and escaped `{{` are ignored.
fn named_path_interpolations(line: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = line;
    while let Some(pos) = rest.find("\"/") {
        rest = &rest[pos + 1..]; // start at the leading '/'
        let end = rest.find('"').unwrap_or(rest.len());
        extract_named(&rest[..end], &mut names);
        rest = &rest[end..];
    }
    names
}

/// Pulls identifier-style `{name}` placeholders out of a single literal.
fn extract_named(literal: &str, names: &mut Vec<String>) {
    let mut rest = literal;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        if let Some(stripped) = rest.strip_prefix('{') {
            rest = stripped; // escaped "{{"
            continue;
        }
        let Some(close) = rest.find('}') else { break };
        let ident: String = rest[..close]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if ident.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            names.push(ident);
        }
        rest = &rest[close + 1..];
    }
}

/// Builds the failure message listing every violation.
fn report(violations: &[String]) -> String {
    format!(
        "unencoded dynamic path segment(s) found — wrap the value with \
         `http::segment(...)` and a positional `{{}}` placeholder, or, if it is \
         a numeric id, add it to NUMERIC_PATH_PARAMS:\n{}",
        violations.join("\n")
    )
}
