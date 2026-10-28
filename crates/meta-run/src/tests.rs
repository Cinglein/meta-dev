use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use super::{EnvPolicy, Environment, Run, RunFailure};

const SHELL: &str = "bash";
const SCRIPT: &str = "-c";
const FED: &str = "cat; echo oops >&2; echo more >&2; exit 3";
const SHOWN: [&str; 3] = ["fed", "oops\nmore", "oops"];
const PATIENT: Duration = Duration::from_secs(5);
const LEADS: &str = "kill -0 -- -$$ 2>/dev/null && echo leader; true";
const LIMIT: Duration = Duration::from_millis(50);
const PROMPT: Duration = Duration::from_secs(1);
const KILLED: [&str; 3] = [
    "set -m; sleep 2 & echo $! >> \"$0\"; echo held",
    "sleep 5 & echo $! >> \"$0\"; wait",
    "sleep 0 & echo $$ >> \"$0\"; exec sleep 5",
];
const PS: [&str; 4] = ["ps", "-o", "stat=", "-p"];
const KILL: [&str; 2] = ["kill", "-KILL"];
const POLL: Duration = Duration::from_millis(2);
const NO_PROMPT: &str = "\"GIT_TERMINAL_PROMPT\": Some(\"0\")";
const INHERITED: [(&str, &str); 4] = [("GIT_DIR", "d"), ("GIT_SSH_COMMAND", "s"), ("LANG", "C"), ("PATH", "/bin")];
const DROPPED: [(EnvPolicy, &str); 4] = [
    (EnvPolicy::Inherit, ""),
    (EnvPolicy::NoRepoVars, "\"GIT_DIR\": None"),
    (EnvPolicy::NoGitVars, "\"GIT_DIR\": None, \"GIT_SSH_COMMAND\": None"),
    (EnvPolicy::Only(&["LANG"]), "\"GIT_DIR\": None, \"GIT_SSH_COMMAND\": None, \"PATH\": None"),
];

struct Scripted;

impl Scripted {
    fn timed_out(script: &'static str, pids: &Path) -> bool {
        let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, script]).arg(pids);
        matches!(run.ran(Instant::now() + LIMIT), Err(RunFailure::TimedOut(_)))
    }

    fn gone(pids: &Path) -> bool {
        let listed = std::fs::read_to_string(pids).unwrap().lines().collect::<Vec<_>>().join(",");
        let live = || Run::new(PS[0], EnvPolicy::Inherit).args(&PS[1..]).arg(&listed).stdout(Instant::now() + PATIENT);
        let until = Instant::now() + PROMPT;
        let polls = std::iter::repeat_with(|| std::thread::sleep(POLL)).take_while(|()| Instant::now() < until);
        polls.into_iter().any(|()| live().unwrap_or_default().lines().all(|state| state.starts_with('Z')))
    }
}

#[test]
fn a_run_feeds_its_input_returns_both_streams_and_stays_in_the_callers_group() {
    let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, FED]).fed(SHOWN[0].to_owned());
    let ran = run.ran(Instant::now() + PATIENT).unwrap();
    assert!(!ran.success() && ran.stdout().text() == SHOWN[0] && ran.stderr().to_string() == SHOWN[1]);
    assert_eq!(ran.succeeded().unwrap_err().to_string(), SHOWN[2]);
    assert_eq!(Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, LEADS]).stdout(Instant::now() + PATIENT).unwrap(), "");
}

#[test]
fn a_run_ends_at_its_deadline_and_kills_its_whole_tree_even_when_a_grandchild_holds_the_pipes() {
    let [stray, killed] = [(); 2].map(|()| tempfile::NamedTempFile::new().unwrap());
    let started = Instant::now();
    let runs = KILLED.map(|script| (script, if script == KILLED[0] { &stray } else { &killed }.path().to_path_buf()));
    let runs = runs.map(|(script, pids)| std::thread::spawn(move || Scripted::timed_out(script, &pids)));
    assert!(runs.into_iter().all(|run| run.join().unwrap()) && started.elapsed() < PROMPT);
    let left = std::fs::read_to_string(stray.path()).unwrap();
    let stopped = Run::new(KILL[0], EnvPolicy::Inherit).args([KILL[1], left.trim()]).ran(Instant::now() + PATIENT);
    assert!(stopped.is_ok() && Scripted::gone(killed.path()) && Scripted::gone(stray.path()));
}

#[test]
fn each_env_policy_drops_only_its_variables_and_never_resets_the_rest() {
    let inherited = INHERITED.map(|(name, value)| (OsString::from(name), Some(OsString::from(value))));
    let inherited = Environment::from(BTreeMap::from(inherited));
    let dropped = DROPPED
        .map(|(policy, _)| format!("{:?}", Environment::from(&Run::inheriting(SHELL, policy, inherited.clone()))));
    assert_eq!(dropped, DROPPED.map(|(_, dropped)| format!("Environment({{{dropped}}})")));
    assert!(format!("{:?}", Environment::from(&Run::git(Path::new(SHELL)))).contains(NO_PROMPT));
}
