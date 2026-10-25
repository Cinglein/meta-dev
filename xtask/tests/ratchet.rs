use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

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

#[derive(Debug)]
struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

#[expect(clippy::expect_used, reason = "a fixture that cannot be built is a failed test")]
impl Fixture {
    fn on_branch() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let origin = directory.path().join("origin.git");
        let work = directory.path().join("work");
        Self::git(directory.path(), &["init", "--quiet", "--bare", &origin.display().to_string()]);
        Self::git(directory.path(), &["init", "--quiet", "--initial-branch", "main", &work.display().to_string()]);
        let fixture = Fixture { _directory: directory, work };
        [(CONFIG, CONFIG_TEXT), (BASELINE, BASELINE_TEXT), (COMMENTED, COMMENTED_TEXT), (CLEAN, CLEAN_TEXT)]
            .into_iter()
            .for_each(|(path, text)| fixture.write(path, text));
        Self::git(&fixture.work, &["add", "--all"]);
        Self::git(
            &fixture.work,
            &["-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "--quiet", "-m", "fixture"],
        );
        Self::git(&fixture.work, &["remote", "add", "origin", &origin.display().to_string()]);
        Self::git(&fixture.work, &["push", "--quiet", "origin", "main"]);
        Self::git(&fixture.work, &["remote", "set-head", "origin", "main"]);
        Self::git(&fixture.work, &["switch", "--quiet", "--create", "change"]);
        fixture
    }

    fn write(&self, path: &str, text: &str) {
        let file = self.work.join(path);
        fs::create_dir_all(file.parent().expect("parent")).expect("create dirs");
        fs::write(file, text).expect("write file");
    }

    fn refusal(&self) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask")).arg("lint").current_dir(&self.work).output();
        let output: Output = output.expect("run xtask");
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    fn git(directory: &Path, args: &[&str]) {
        let status =
            Command::new("git").args(["-c", "core.hooksPath=/dev/null"]).args(args).current_dir(directory).status();
        status.expect("run git").success().then_some(()).expect("git succeeds");
    }
}

#[test]
fn a_raised_or_deleted_baseline_fails() {
    let fixture = Fixture::on_branch();
    fixture.write(COMMENTED, TWICE_COMMENTED);
    fixture.write(BASELINE, RAISED_BASELINE);
    assert!(fixture.refusal().contains(RAISED));
    fs::remove_file(fixture.work.join(BASELINE)).expect("remove baseline");
    assert!(fixture.refusal().contains(OVER));
}

#[test]
fn no_files_fails() {
    let fixture = Fixture::on_branch();
    fs::remove_dir_all(fixture.work.join("crates")).expect("remove sources");
    assert!(fixture.refusal().contains(NO_FILES));
}

#[test]
fn a_moved_file_with_the_same_violations_passes() {
    let fixture = Fixture::on_branch();
    fixture.write(MOVED, COMMENTED_TEXT);
    fs::remove_file(fixture.work.join(COMMENTED)).expect("remove original");
    assert!(fixture.refusal().is_empty());
}
