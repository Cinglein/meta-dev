use std::ffi::OsStr;
use std::io::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::Run;

use super::Host;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::project::Project;
use crate::request::tests::pids::Pids;

const IDENTITY: [&str; 4] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid"];
const SETUP: [&[&str]; 2] = [&["init", "-q", "-b", "main"], &["commit", "-q", "--allow-empty", "-m", "start"]];
const HANGING: &str =
    "[remote \"origin\"]\n\turl = ssh://meta.invalid/x\n[core]\n\tsshCommand = bash -c 'echo $$ >> $0 && sleep 2'";
const CONFIG: &str = "remote = \"origin\"\n[ci]\ngit_millis = 100\n";
const LISTING: [&str; 2] = ["ls-remote", "origin"];
const PROMPT: Duration = Duration::from_secs(1);

struct Hanging;

impl Hanging {
    fn repo(helpers: &Path) -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        let set_up =
            SETUP.map(|words| Run::git(repo.path()).args(IDENTITY).args(words).stdout(Instant::now() + PROMPT));
        set_up.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
        let config = std::fs::OpenOptions::new().append(true).open(repo.path().join(".git/config"));
        writeln!(config.unwrap(), "{HANGING} {}", helpers.display()).unwrap();
        std::fs::write(repo.path().join("meta.toml"), CONFIG).unwrap();
        repo
    }
}

#[test]
fn a_push_and_a_gc_ls_remote_to_a_hanging_remote_give_up_at_the_deadline_and_leave_no_helper() {
    let helpers = tempfile::NamedTempFile::new().unwrap();
    let repo = Hanging::repo(helpers.path());
    let project = Project::try_from(repo.path()).unwrap();
    let call = GitCall::common(&repo.path().join(".git"), &LISTING.map(OsStr::new)).reaching();
    let started = Instant::now();
    let listed = std::thread::scope(|threads| {
        let listed = threads.spawn(|| Host::new(project.ci_settings().clone()).git(&call).is_err());
        project.push_failure().is_some() && listed.join().unwrap()
    });
    assert!(listed && started.elapsed() < PROMPT && Pids::gone(helpers.path(), &LISTING));
}
