mod fixture;

use std::fs;

use fixture::Fixture;

const CONFIG: &str = "xtask.toml";
const CONFIG_TEXT: &str = "surfaces = [\"xtask.toml\", \"lint-baseline.toml\"]\n[lints]\nbaseline = \"lint-baseline.toml\"\n\n[ci]\nstep = []\n";
const NARROWED_CONFIG: &str = "surfaces = []\n[lints]\nbaseline = \"lint-baseline.toml\"\n\n[ci]\nstep = []\n";
const BASELINE: &str = "lint-baseline.toml";
const RAISED_BASELINE: &str = "[no-comments]\n\"crates/a/src/lib.rs\" = 1\n";
const SOURCE: &str = "crates/a/src/lib.rs";
const SOURCE_TEXT: &str = "pub struct Alpha;\n";
const CHANGED_SOURCE: &str = "pub struct Beta;\n";
const LISTED: &str = "!! enforcement surfaces changed since the merge base:";
const CI: &str = "ci";
const SUMMARY: &str = "summary.md";

#[test]
fn a_changed_surface_is_listed_without_failing() {
    let fixture = Fixture::new();
    fixture.write(CONFIG, CONFIG_TEXT).write(BASELINE, "").write(SOURCE, SOURCE_TEXT).branch();
    let summary = fixture.path(SUMMARY);
    fixture.write(BASELINE, RAISED_BASELINE);
    let listed = format!("\n{LISTED}\n- {BASELINE}\n");
    let output = fixture.xtask(CI, Some(&summary));
    assert!(output.status.success() && String::from_utf8_lossy(&output.stdout).contains(&listed));
    assert_eq!(fs::read_to_string(summary).ok(), Some(listed));
}

#[test]
fn other_changes_list_nothing() {
    let fixture = Fixture::new();
    fixture.write(CONFIG, CONFIG_TEXT).write(BASELINE, "").write(SOURCE, SOURCE_TEXT).branch();
    fixture.write(SOURCE, CHANGED_SOURCE);
    let output = fixture.xtask(CI, None);
    assert!(output.status.success() && !String::from_utf8_lossy(&output.stdout).contains(LISTED));
}

#[test]
fn a_narrowed_surface_list_still_lists_the_dropped_surfaces() {
    let fixture = Fixture::new();
    fixture.write(CONFIG, CONFIG_TEXT).write(BASELINE, "").write(SOURCE, SOURCE_TEXT).branch();
    fixture.write(CONFIG, NARROWED_CONFIG);
    fixture.write(BASELINE, RAISED_BASELINE);
    let listed = format!("- {BASELINE}\n- {CONFIG}\n");
    let output = fixture.xtask(CI, None);
    assert!(output.status.success() && String::from_utf8_lossy(&output.stdout).contains(&listed));
}
