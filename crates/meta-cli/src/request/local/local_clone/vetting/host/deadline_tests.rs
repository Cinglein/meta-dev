use std::ffi::OsStr;
use std::io::Write as _;
use std::process::Command;
use std::time::{Duration, Instant};

use super::Host;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::project::Project;

const IDENTITY: [&str; 4] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid"];
const SETUP: [&[&str]; 2] = [&["init", "-q", "-b", "main"], &["commit", "-q", "--allow-empty", "-m", "start"]];
const HANGING: &str = "[remote \"origin\"]\n\turl = ssh://meta.invalid/x\n[core]\n\tsshCommand = bash -c 'sleep 2' x\n";
const CONFIG: &str = "remote = \"origin\"\n[ci]\ngit_millis = 100\n";
const LISTING: [&str; 2] = ["ls-remote", "origin"];
const PROMPT: Duration = Duration::from_secs(1);

struct Hanging(tempfile::TempDir);

impl Hanging {
    fn new() -> Self {
        let repo = Self(tempfile::tempdir().unwrap());
        let set_up =
            SETUP.map(|words| Command::new("git").arg("-C").arg(repo.0.path()).args(IDENTITY).args(words).status());
        assert!(set_up.iter().all(|status| status.as_ref().is_ok_and(std::process::ExitStatus::success)));
        let config = std::fs::OpenOptions::new().append(true).open(repo.0.path().join(".git/config"));
        config.unwrap().write_all(HANGING.as_bytes()).unwrap();
        std::fs::write(repo.0.path().join("meta.toml"), CONFIG).unwrap();
        repo
    }
}

#[test]
fn a_push_and_a_gc_ls_remote_to_a_hanging_remote_give_up_at_the_deadline() {
    let repo = Hanging::new();
    let started = Instant::now();
    let pushed = Project::try_from(repo.0.path()).unwrap().push_failure();
    let pushed_in = started.elapsed();
    let call = GitCall::common(&repo.0.path().join(".git"), &LISTING.map(OsStr::new));
    let listed = Host::default().git(&call);
    assert!(pushed.is_some() && pushed_in < PROMPT);
    assert!(listed.is_err() && started.elapsed() < PROMPT + PROMPT);
}
