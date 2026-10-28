use std::process::Command;
use std::time::{Duration, Instant};

use super::Project;
use crate::request::ci::Commit;

const REMOTE: &str = "origin";
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn a_project_serves_its_meta_toml_and_counts_commits_the_remote_does_not_list_as_pushed() {
    let dir = tempfile::tempdir().unwrap();
    Command::new("git").arg("init").arg("-q").arg(dir.path()).status().unwrap();
    std::fs::write(dir.path().join("meta.toml"), format!("remote = \"{REMOTE}\"")).unwrap();
    let project = Project::try_from(dir.path()).unwrap();
    let Ok(commit): Result<Commit, _> = COMMIT.parse();
    let pushed = project.pushed_among(std::slice::from_ref(&commit), &[], Instant::now() + Duration::from_secs(30));
    assert_eq!((project.config().remote().unwrap().to_string(), pushed), (REMOTE.to_owned(), vec![commit]));
}
