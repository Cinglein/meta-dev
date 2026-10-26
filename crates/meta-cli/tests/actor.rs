#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

const META: &str = env!("CARGO_BIN_EXE_meta");
const IDENTITY: [&str; 6] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid", "-c", "init.defaultBranch=main"];
const ISSUE: [&str; 2] = ["issue", "text"];
const LOCAL_NAME: &str = "localname";
const OWNER: &str = "owner";
const ACTOR_INDEX: usize = 2;

struct Git;

impl Git {
    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(IDENTITY).args(words).output().unwrap()
    }
}

#[test]
fn an_unset_actor_is_written_as_owner_never_the_local_user() {
    let origin = tempfile::tempdir().unwrap();
    let clone = tempfile::tempdir().unwrap();
    Git::run(origin.path(), &["init", "-q", "--bare"]);
    Git::run(clone.path(), &["init", "-q"]);
    Git::run(clone.path(), &["remote", "add", "origin", origin.path().to_str().unwrap()]);
    std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
    let output = Command::new(META)
        .current_dir(clone.path())
        .args(ISSUE)
        .env_remove("META_ACTOR")
        .env("USER", LOCAL_NAME)
        .output()
        .unwrap();
    let written = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(written.split_whitespace().nth(ACTOR_INDEX), Some(OWNER));
}
