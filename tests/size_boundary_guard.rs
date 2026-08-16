//! Source guard: every file outside the function-cap boundary declares it.
//!
//! CLAUDE.md rule 2 scopes the 20-line function cap to `src/`, but
//! `clippy::too_many_lines` is configured crate-wide — the only thing holding
//! it back in `tests/` and `examples/` is a file-level
//! `#![allow(clippy::too_many_lines)]`. Without this guard the boundary is
//! convention only: a new integration test with a perfectly ordinary 30-line
//! arrange/act/assert body fails CI with `too_many_lines`, reporting a
//! violation of a rule that does not apply there.
//!
//! So the omission is caught here instead, with a message that says what to
//! add, and it is caught as soon as the file is created rather than whenever
//! one of its functions happens to cross 20 lines.

// CLAUDE.md rule 2 scopes the 20-line function cap to src/; integration
// tests sit outside that boundary, so a linear arrange/assert case may
// run long rather than be split for the sake of the count.
#![allow(clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};

/// The attribute every `tests/` and `examples/` file must carry.
const MARKER: &str = "#![allow(clippy::too_many_lines)]";

/// Trees that sit outside the 20-line function cap.
const EXEMPT_TREES: &[&str] = &["tests", "examples"];

#[test]
fn files_outside_the_function_cap_declare_the_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut missing = Vec::new();
    let mut checked = 0;

    for tree in EXEMPT_TREES {
        let dir = root.join(tree);
        let mut files = Vec::new();
        collect_rs_files(&dir, &mut files);
        assert!(!files.is_empty(), "no .rs files under {}", dir.display());
        checked += files.len();
        missing.extend(files.into_iter().filter(|file| !declares_boundary(file)));
    }

    assert!(checked > 0, "guard scanned nothing");
    assert!(missing.is_empty(), "{}", report(&missing, root));
}

/// Whether `file` carries the boundary marker.
fn declares_boundary(file: &Path) -> bool {
    let text = fs::read_to_string(file).expect("source file is readable");
    text.lines().any(|line| line.trim() == MARKER)
}

/// Collects every `.rs` file under `dir`, recursively.
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("exempt tree is readable");
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Builds the failure message, naming each file and the fix.
fn report(missing: &[PathBuf], root: &Path) -> String {
    let names: Vec<String> = missing
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect();
    format!(
        "these files sit outside CLAUDE.md rule 2's 20-line function cap but do \
         not say so, and would fail CI with a misleading `too_many_lines` \
         instead. Add `{MARKER}` below the module docs:\n  {}",
        names.join("\n  ")
    )
}
