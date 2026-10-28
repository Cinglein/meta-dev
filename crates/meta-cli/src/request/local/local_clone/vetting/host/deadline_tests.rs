use std::ffi::OsStr;
use std::io::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::{EnvPolicy, Run};

use super::Host;
use crate::request::local::local_clone::vetting::git_call::GitCall;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::project::Project;

const IDENTITY: [&str; 4] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid"];
const SETUP: [&[&str]; 2] = [&["init", "-q", "-b", "main"], &["commit", "-q", "--allow-empty", "-m", "start"]];
const HANGING: &str =
    "[remote \"origin\"]\n\turl = ssh://meta.invalid/x\n[core]\n\tsshCommand = bash -c 'echo $$ >> $0 && sleep 2'";
const CONFIG: &str = "remote = \"origin\"\n[ci]\ngit_millis = 100\n";
const LISTING: [&str; 2] = ["ls-remote", "origin"];
const PROMPT: Duration = Duration::from_secs(1);
const ALIVE: [&str; 2] = ["kill", "-0"];
const POLL: Duration = Duration::from_millis(10);

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

    fn gone(helpers: &Path) -> bool {
        let pids = std::fs::read_to_string(helpers).unwrap_or_default();
        let alive =
            |pid: &str| Run::new(ALIVE[0], EnvPolicy::Inherit).args([ALIVE[1], pid]).stdout(Instant::now() + PROMPT);
        let polls = PROMPT.as_millis() / POLL.as_millis();
        pids.lines().count() == LISTING.len()
            && (0..polls).any(|_| {
                std::thread::sleep(POLL);
                !pids.lines().any(|pid| alive(pid).is_ok())
            })
    }
}

#[test]
fn a_push_and_a_gc_ls_remote_to_a_hanging_remote_give_up_at_the_deadline_and_leave_no_helper() {
    let helpers = tempfile::NamedTempFile::new().unwrap();
    let repo = Hanging::repo(helpers.path());
    let project = Project::try_from(repo.path()).unwrap();
    let call = GitCall::common(&repo.path().join(".git"), &LISTING.map(OsStr::new));
    let started = Instant::now();
    let listed = std::thread::scope(|threads| {
        let listed = threads.spawn(|| Host::new(project.ci_settings().clone()).git(&call).is_err());
        project.push_failure().is_some() && listed.join().unwrap()
    });
    assert!(listed && started.elapsed() < PROMPT && Hanging::gone(helpers.path()));
}
