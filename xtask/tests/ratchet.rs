mod fixture;

use std::fs;

use fixture::Fixture;

const CONFIG: &str = "xtask.toml";
const CONFIG_TEXT: &str = "surfaces = [\"xtask.toml\", \"lint-baseline.toml\"]\n[lints]\nbaseline = \"lint-baseline.toml\"\nproduct = [\"crates\"]\nexclude = [\".git\"]\n\n[ci]\nstep = []\n";
const BASELINE: &str = "lint-baseline.toml";
const BASELINE_TEXT: &str = "[no-comments]\n\"crates/a/src/lib.rs\" = 1\n";
const RAISED_BASELINE: &str = "[no-comments]\n\"crates/a/src/lib.rs\" = 2\n";
const COMMENTED: &str = "crates/a/src/lib.rs";
const COMMENTED_TEXT: &str = "pub struct Alpha;\n// one\n";
const TWICE_COMMENTED: &str = "pub struct Alpha;\n// one\n// two\n";
const CLEAN: &str = "crates/b/src/lib.rs";
const CLEAN_TEXT: &str = "pub struct Beta;\n";
const MOVED: &str = "crates/c/src/lib.rs";
const RAISED: &str = "no-comments: 2 found, baseline 1";
const OVER: &str = "no-comments: 2 found, baseline 0";
const NO_FILES: &str = "no rust files to lint";
const LINT: &str = "lint";

#[test]
fn a_raised_or_deleted_baseline_fails() {
    let fixture = Fixture::new();
    fixture
        .write(CONFIG, CONFIG_TEXT)
        .write(BASELINE, BASELINE_TEXT)
        .write(COMMENTED, COMMENTED_TEXT)
        .write(CLEAN, CLEAN_TEXT)
        .branch();
    fixture.write(COMMENTED, TWICE_COMMENTED);
    fixture.write(BASELINE, RAISED_BASELINE);
    assert!(String::from_utf8_lossy(&fixture.xtask(LINT, None).stderr).contains(RAISED));
    fs::remove_file(fixture.path(BASELINE)).expect("remove baseline");
    assert!(String::from_utf8_lossy(&fixture.xtask(LINT, None).stderr).contains(OVER));
}

#[test]
fn no_files_fails() {
    let fixture = Fixture::new();
    fixture
        .write(CONFIG, CONFIG_TEXT)
        .write(BASELINE, BASELINE_TEXT)
        .write(COMMENTED, COMMENTED_TEXT)
        .write(CLEAN, CLEAN_TEXT)
        .branch();
    fs::remove_dir_all(fixture.path("crates")).expect("remove sources");
    assert!(String::from_utf8_lossy(&fixture.xtask(LINT, None).stderr).contains(NO_FILES));
}

#[test]
fn a_moved_file_with_the_same_violations_passes() {
    let fixture = Fixture::new();
    fixture
        .write(CONFIG, CONFIG_TEXT)
        .write(BASELINE, BASELINE_TEXT)
        .write(COMMENTED, COMMENTED_TEXT)
        .write(CLEAN, CLEAN_TEXT)
        .branch();
    fixture.write(MOVED, COMMENTED_TEXT);
    fs::remove_file(fixture.path(COMMENTED)).expect("remove original");
    assert!(String::from_utf8_lossy(&fixture.xtask(LINT, None).stderr).is_empty());
}
