//! Golden plan tests: each `tests/golden/<case>/space.toml` is planned inside
//! and outside the backend, and compared with `expected.txt` and
//! `expected.json`. `UPDATE_GOLDEN=1 cargo test` rewrites them.

use std::fs;
use std::path::Path;

use gogo_spaces::plan::{plan, Options};
use gogo_spaces::space::{parse, Source};

#[test]
fn plans_match_golden_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut cases: Vec<_> = fs::read_dir(&root).unwrap().map(|e| e.unwrap().path()).collect();
    cases.sort();
    assert!(!cases.is_empty(), "no golden cases in {}", root.display());

    let mut failures = Vec::new();
    for case in cases {
        let file = case.join("space.toml");
        let text = fs::read_to_string(&file).unwrap();
        // A fixed file path keeps the output the same on every machine.
        let name = case.file_name().unwrap().to_string_lossy();
        let fixed = format!("/spaces/{name}.toml");
        let space = parse(&text, Path::new(&fixed), Source::User, Path::new("/cwd"))
            .unwrap_or_else(|e| panic!("{}: {e}", file.display()));

        let inside = plan(&space, "herdr", Options { inside: true, attach: true, focus: true });
        let outside = plan(&space, "herdr", Options { inside: false, attach: true, focus: true });
        let actual_txt = format!(
            "# inside the backend\n{}\n# outside it\n{}",
            inside.render(),
            outside.render()
        );
        let actual_json = serde_json::to_string_pretty(&outside).unwrap() + "\n";

        for (name, actual) in [("expected.txt", actual_txt), ("expected.json", actual_json)] {
            let path = case.join(name);
            if update {
                fs::write(&path, &actual).unwrap();
            } else if fs::read_to_string(&path).ok().as_deref() != Some(actual.as_str()) {
                failures.push(format!("{}\n--- actual ---\n{actual}", path.display()));
            }
        }
    }
    assert!(failures.is_empty(), "golden mismatch (UPDATE_GOLDEN=1 to accept):\n\n{}", failures.join("\n\n"));
}
