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
const LIMIT: Duration = Duration::from_millis(100);
const PROMPT: Duration = Duration::from_secs(1);
const HELD_ELSEWHERE: &str = "set -m; sleep 2 & echo held";
const KILLED: [(&str, KillScope, &str); 2] = [
    ("sleep 5 & echo $! > \"$0\"", KillScope::Group, "group"),
    ("echo $$ > \"$0\"; exec sleep 5", KillScope::Process, "process"),
];
const KILL: &str = "kill";
const ALIVE: &str = "-0";
const POLLS: usize = 50;
const POLL: Duration = Duration::from_millis(10);
const ENV: &str = "/usr/bin/env";
const INHERITED: [(&str, &str); 4] = [("GIT_DIR", "d"), ("GIT_SSH_COMMAND", "s"), ("LANG", "C"), ("PATH", "/bin")];
const KEPT: [(EnvPolicy, &str); 4] = [
    (EnvPolicy::Inherit, "GIT_DIR GIT_SSH_COMMAND LANG PATH"),
    (EnvPolicy::NoRepoVars, "GIT_SSH_COMMAND LANG PATH"),
    (EnvPolicy::NoGitVars, "LANG PATH"),
    (EnvPolicy::Only(&["LANG"]), "LANG"),
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
fn a_run_feeds_its_input_and_returns_its_status_and_both_streams() {
    let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, FED]).fed(INPUT.to_owned());
    let ran = run.ran(Instant::now() + PATIENT).unwrap();
    assert!(!ran.success() && ran.stdout().text() == INPUT && ran.stderr().to_string() == SHOWN[0]);
    assert_eq!(ran.succeeded().unwrap_err().to_string(), SHOWN[1]);
}

#[test]
fn a_run_ends_at_its_deadline_and_kills_its_scope_even_when_a_grandchild_holds_the_pipes() {
    let dir = tempfile::tempdir().unwrap();
    let started = Instant::now();
    let elsewhere = Scripted::timed_out(HELD_ELSEWHERE, KillScope::Group, dir.path());
    assert!(elsewhere && started.elapsed() < PROMPT);
    let files = KILLED.map(|(.., name)| dir.path().join(name));
    let killed = KILLED.iter().zip(&files).map(|((script, scope, _), file)| Scripted::timed_out(script, *scope, file));
    assert!(killed.collect::<Vec<_>>() == [true, true] && files.iter().all(|file| Scripted::gone(file)));
}

#[test]
fn each_env_policy_passes_on_only_its_variables() {
    let shown = KEPT.map(|(policy, _)| {
        let inherited = INHERITED.map(|(name, value)| (OsString::from(name), OsString::from(value)));
        let run = Run::inheriting(ENV, policy, Environment::from(BTreeMap::from(inherited)));
        let printed = run.stdout(Instant::now() + PATIENT).unwrap();
        let mut names: Vec<&str> =
            printed.lines().filter_map(|line| line.split_once('=')).map(|(name, _)| name).collect();
        names.sort_unstable();
        names.join(" ")
    });
    assert_eq!(shown, KEPT.map(|(_, kept)| kept.to_owned()));
}
