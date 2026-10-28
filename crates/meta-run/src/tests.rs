use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{EnvPolicy, Environment, KillScope, Run, RunFailure};

const SHELL: &str = "bash";
const SCRIPT: &str = "-c";
const FED: &str = "cat; echo oops >&2; echo more >&2; exit 3";
const SHOWN: [&str; 3] = ["fed", "oops\nmore", "oops"];
const PATIENT: Duration = Duration::from_secs(5);
const LEADS: &str = "[ \"$(ps -o pgid= -p $$ | tr -d ' ')\" = \"$$\" ] && echo leader; true";
const LED: [(Option<KillScope>, &str); 2] = [(Some(KillScope::Group), "leader\n"), (None, "")];
const LIMIT: Duration = Duration::from_millis(100);
const PROMPT: Duration = Duration::from_secs(1);
const KILLED: [(&str, KillScope, &str); 3] = [
    ("set -m; sleep 2 & echo $! > \"$0\"; echo held", KillScope::Group, "elsewhere"),
    ("sleep 5 & echo $! > \"$0\"", KillScope::Group, "group"),
    ("echo $$ > \"$0\"; exec sleep 5", KillScope::Process, "process"),
];
const KILL: [&str; 3] = ["kill", "-0", "-KILL"];
const POLL: Duration = Duration::from_millis(10);
const INHERITED: [(&str, &str); 4] = [("GIT_DIR", "d"), ("GIT_SSH_COMMAND", "s"), ("LANG", "C"), ("PATH", "/bin")];
const DROPPED: [(EnvPolicy, &str); 4] = [
    (EnvPolicy::Inherit, ""),
    (EnvPolicy::NoRepoVars, "\"GIT_DIR\": None"),
    (EnvPolicy::NoGitVars, "\"GIT_DIR\": None, \"GIT_SSH_COMMAND\": None"),
    (EnvPolicy::Only(&["LANG"]), "\"GIT_DIR\": None, \"GIT_SSH_COMMAND\": None, \"PATH\": None"),
];

struct Scripted;

impl Scripted {
    fn timed_out(script: &str, scope: KillScope, file: &Path) -> bool {
        let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, script]).arg(file).scoped(scope);
        matches!(run.ran(Instant::now() + LIMIT), Err(RunFailure::TimedOut(_)))
    }

    fn signalled(file: &Path, signal: &str) -> bool {
        let pid = std::fs::read_to_string(file).unwrap();
        let kill = Run::new(KILL[0], EnvPolicy::Inherit).args([signal, pid.trim()]);
        kill.ran(Instant::now() + PATIENT).unwrap().success()
    }

    fn gone(file: &Path) -> bool {
        let polls = (0..PROMPT.as_millis() / POLL.as_millis()).map(|_| std::thread::sleep(POLL));
        polls.into_iter().any(|()| !Self::signalled(file, KILL[1]))
    }
}

#[test]
fn a_run_feeds_its_input_returns_both_streams_and_leads_its_own_group_only_when_asked() {
    let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, FED]).fed(SHOWN[0].to_owned());
    let ran = run.ran(Instant::now() + PATIENT).unwrap();
    assert!(!ran.success() && ran.stdout().text() == SHOWN[0] && ran.stderr().to_string() == SHOWN[1]);
    assert_eq!(ran.succeeded().unwrap_err().to_string(), SHOWN[2]);
    let led = LED.map(|(scope, _)| {
        let run = scope.into_iter().fold(Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, LEADS]), Run::scoped);
        run.stdout(Instant::now() + PATIENT).unwrap()
    });
    assert_eq!(led, LED.map(|(_, shown)| shown.to_owned()));
}

#[test]
fn a_run_ends_at_its_deadline_and_kills_its_scope_even_when_a_grandchild_holds_the_pipes() {
    let dir = tempfile::tempdir().unwrap();
    let files = KILLED.map(|(.., name)| dir.path().join(name));
    let started = Instant::now();
    let run = |((script, scope, _), file): ((&'static str, KillScope, &str), PathBuf)| {
        std::thread::spawn(move || Scripted::timed_out(script, scope, &file))
    };
    let runs: Vec<_> = KILLED.into_iter().zip(files.clone()).map(run).collect();
    assert!(runs.into_iter().all(|run| run.join().unwrap()) && started.elapsed() < PROMPT);
    assert!(Scripted::signalled(&files[0], KILL[2]) && files.iter().all(|file| Scripted::gone(file)));
}

#[test]
fn each_env_policy_drops_only_its_variables_and_never_resets_the_rest() {
    let inherited = INHERITED.map(|(name, value)| (OsString::from(name), Some(OsString::from(value))));
    let inherited = Environment::from(BTreeMap::from(inherited));
    let dropped = DROPPED
        .map(|(policy, _)| format!("{:?}", Environment::from(&Run::inheriting(SHELL, policy, inherited.clone()))));
    assert_eq!(dropped, DROPPED.map(|(_, dropped)| format!("Environment({{{dropped}}})")));
}
