use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use super::{EnvPolicy, Environment, KillScope, Run, RunFailure};

const SHELL: &str = "bash";
const SCRIPT: &str = "-c";
const FED: &str = "cat; echo oops >&2; echo more >&2; exit 3";
const INPUT: &str = "fed";
const SHOWN: [&str; 2] = ["oops\nmore", "oops"];
const PATIENT: Duration = Duration::from_secs(5);
const LEADS: &str = "[ \"$(ps -o pgid= -p $$ | tr -d ' ')\" = \"$$\" ] && echo leader; true";
const LED: [(KillScope, &str); 2] = [(KillScope::Group, "leader\n"), (KillScope::Process, "")];
const LIMIT: Duration = Duration::from_millis(100);
const PROMPT: Duration = Duration::from_secs(1);
const KILLED: [(&str, KillScope, &str); 3] = [
    ("set -m; sleep 2 & echo held", KillScope::Group, "elsewhere"),
    ("sleep 5 & echo $! > \"$0\"", KillScope::Group, "group"),
    ("echo $$ > \"$0\"; exec sleep 5", KillScope::Process, "process"),
];
const KILL: &str = "kill";
const ALIVE: &str = "-0";
const POLLS: usize = 50;
const POLL: Duration = Duration::from_millis(10);
const ENV: &str = "/usr/bin/env";
const INHERITED: [(&str, &str); 4] = [("GIT_DIR", "d"), ("GIT_SSH_COMMAND", "s"), ("LANG", "C"), ("PATH", "/bin")];
const DROPPED: [(EnvPolicy, &[&str]); 4] = [
    (EnvPolicy::Inherit, &[]),
    (EnvPolicy::NoRepoVars, &["GIT_DIR"]),
    (EnvPolicy::NoGitVars, &["GIT_DIR", "GIT_SSH_COMMAND"]),
    (EnvPolicy::Only(&["LANG"]), &["GIT_DIR", "GIT_SSH_COMMAND", "PATH"]),
];

struct Scripted;

impl Scripted {
    fn timed_out(script: &str, scope: KillScope, file: &Path) -> bool {
        let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, script]).arg(file).scoped(scope);
        matches!(run.ran(Instant::now() + LIMIT), Err(RunFailure::TimedOut(_)))
    }

    fn gone(file: &Path) -> bool {
        let pid = std::fs::read_to_string(file).unwrap();
        (0..POLLS).any(|_| {
            std::thread::sleep(POLL);
            let alive = Run::new(KILL, EnvPolicy::Inherit).args([ALIVE, pid.trim()]).ran(Instant::now() + PATIENT);
            alive.is_ok_and(|ran| !ran.success())
        })
    }
}

#[test]
fn a_run_feeds_its_input_returns_both_streams_and_leads_its_own_group_only_in_group_scope() {
    let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, FED]).fed(INPUT.to_owned());
    let ran = run.ran(Instant::now() + PATIENT).unwrap();
    assert!(!ran.success() && ran.stdout().text() == INPUT && ran.stderr().to_string() == SHOWN[0]);
    assert_eq!(ran.succeeded().unwrap_err().to_string(), SHOWN[1]);
    let led = LED.map(|(scope, _)| {
        let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, LEADS]).scoped(scope);
        run.stdout(Instant::now() + PATIENT).unwrap()
    });
    assert_eq!(led, LED.map(|(_, shown)| shown.to_owned()));
}

#[test]
fn a_run_ends_at_its_deadline_and_kills_its_scope_even_when_a_grandchild_holds_the_pipes() {
    let dir = tempfile::tempdir().unwrap();
    let files = KILLED.map(|(.., name)| dir.path().join(name));
    let started = Instant::now();
    let runs = KILLED.into_iter().zip(files.clone());
    let runs: Vec<_> = runs
        .map(|((script, scope, _), file)| std::thread::spawn(move || Scripted::timed_out(script, scope, &file)))
        .collect();
    let timed_out: Vec<bool> = runs.into_iter().map(|run| run.join().unwrap()).collect();
    assert!(timed_out == [true, true, true] && started.elapsed() < PROMPT);
    assert!(files.iter().skip(1).all(|file| Scripted::gone(file)));
}

#[test]
fn each_env_policy_drops_only_its_variables_and_never_resets_the_rest() {
    let inherited = INHERITED.map(|(name, value)| (OsString::from(name), Some(OsString::from(value))));
    let inherited = Environment::from(BTreeMap::from(inherited));
    let dropped = DROPPED.map(|(policy, _)| Environment::from(&Run::inheriting(ENV, policy, inherited.clone())));
    let expected = DROPPED.map(|(_, names)| {
        Environment::from(names.iter().map(|name| (OsString::from(name), None)).collect::<BTreeMap<_, _>>())
    });
    assert_eq!(dropped, expected);
}
