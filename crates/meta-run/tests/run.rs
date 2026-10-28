#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::{Budget, EnvPolicy, Environment, Ran, Run, RunFailure};

const SHELL: &str = "bash";
const LEADS: &str = "kill -0 -- -$$ 2>/dev/null && echo leader; true";
const LEADER: &str = "leader\n";
const PATIENT: Duration = Duration::from_secs(5);
const SCRIPT: &str = "-c";
const FED: &str = "cat; echo oops >&2; echo more >&2; exit 3";
const SHOWN: [&str; 2] = ["fed", "oops\nmore"];
const COMPLAINT: &str = "oops";
const LIMIT: Duration = Duration::from_millis(60);
const PROMPT: Duration = Duration::from_secs(1);
const KILLED: [&str; 4] = [
    "sleep 5 & echo $! >> \"$0\"; echo $$ >> \"$0\"; echo held",
    "bash -c 'trap \"\" TERM; echo $$ >> \"$0\"; exec sleep 5' \"$0\" & echo $$ >> \"$0\"; wait",
    "trap '' TERM; echo $$ >> \"$0\"; for _ in 1 2 3 4 5; do sleep 1 & echo $! >> \"$0\"; wait; done",
    "echo $$ >> \"$0\"; kill -STOP $$",
];
const NESTED: &str = "set -m; (trap '' TERM; exec sh -c 'echo $$ >> \"$0\"; exec sleep 5' \"$0\") & inner=$!; \
     trap 'kill -TERM -$inner; sleep 0.05; kill -KILL -$inner; exit' TERM; echo $$ >> \"$0\"; wait";
const RECORDED_LEN: usize = 9;
const EXPIRED: &str = "deadline had passed";
const STALLS: &str = "waiting for terminal input";
const PS: [&str; 4] = ["ps", "-o", "stat=", "-p"];
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
    fn shell(script: &str) -> Run {
        Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, script])
    }

    fn timed_out(ran: Result<Ran, RunFailure>) -> String {
        ran.err().map(|failure| failure.to_string()).unwrap_or_default()
    }

    fn gone(pids: &Path) -> bool {
        let listed: Vec<String> =
            std::fs::read_to_string(pids).unwrap_or_default().split_whitespace().map(str::to_owned).collect();
        let until = Instant::now() + PROMPT;
        let ps = || Run::new(PS[0], EnvPolicy::Inherit).args(&PS[1..]).arg(listed.join(",")).stdout(until + PATIENT);
        let polls = std::iter::repeat_with(|| std::thread::sleep(POLL)).take_while(|()| Instant::now() < until);
        let zombies = |states: String| states.lines().all(|state| state.starts_with('Z'));
        listed.len() >= RECORDED_LEN && polls.into_iter().any(|()| zombies(ps().unwrap_or_default()))
    }
}

#[test]
fn a_run_feeds_its_input_returns_both_streams_and_leads_its_own_group() {
    let ran = Scripted::shell(FED).fed(SHOWN[0].to_owned()).ran(Instant::now() + PATIENT).unwrap();
    assert!(!ran.success() && ran.stdout().text() == SHOWN[0] && ran.stderr().to_string() == SHOWN[1]);
    let reported = Scripted::shell(FED).fed(SHOWN[0].to_owned()).reported(Instant::now() + PATIENT);
    let complained = |shown: String| shown == COMPLAINT;
    assert!(
        ran.succeeded().is_err_and(|ran| complained(ran.to_string()))
            && reported.is_err_and(|unread| complained(unread.to_string()))
    );
    let led = Scripted::shell(LEADS).reported(Instant::now() + PATIENT).map(|out| out.text().into_owned());
    assert!(
        [led.ok(), Scripted::shell(LEADS).stdout(Instant::now() + PATIENT).ok()]
            .iter()
            .all(|led| led.as_deref() == Some(LEADER))
    );
}

#[test]
fn a_run_ends_at_its_deadline_and_kills_its_whole_group_holders_deaf_respawning_stalled_and_nested_alike() {
    let killed = tempfile::NamedTempFile::new().unwrap();
    let started = Instant::now();
    let runs = KILLED.map(|script| Scripted::shell(script).arg(killed.path()));
    let runs = runs.into_iter().chain([Run::nested(SHELL).args([SCRIPT, NESTED]).arg(killed.path())]);
    let runs = runs.map(|run| std::thread::spawn(move || Scripted::timed_out(run.ran(Instant::now() + LIMIT))));
    let runs: Vec<_> = runs.collect();
    assert!(runs.into_iter().all(|run| run.join().unwrap().contains(STALLS)) && started.elapsed() < PROMPT);
    assert!(Scripted::gone(killed.path()), "{:?}", std::fs::read_to_string(killed.path()));
}

#[test]
fn each_env_policy_drops_only_its_variables_and_a_budget_caps_calls_and_rests_at_its_overall_deadline() {
    let inherited = INHERITED.map(|(name, value)| (OsString::from(name), Some(OsString::from(value))));
    let inherited = Environment::from(BTreeMap::from(inherited));
    let dropped = DROPPED
        .map(|(policy, _)| format!("{:?}", Environment::from(&Run::inheriting(SHELL, policy, inherited.clone()))));
    assert_eq!(dropped, DROPPED.map(|(_, dropped)| format!("Environment({{{dropped}}})")));
    let git = Run::git(Path::new(SHELL));
    let shown = format!("{:?}", Environment::from(&git));
    assert!(shown.contains(NO_PROMPT) && git.ran(Instant::now()).is_err_and(|late| late.to_string().contains(EXPIRED)));
    let started = Instant::now();
    Budget::new(PATIENT).within(started + POLL).rest(PATIENT);
    let rested = started.elapsed();
    assert!((POLL..PATIENT).contains(&rested) && Budget::new(PATIENT).within(started).deadline() == started);
}

impl std::fmt::Display for Scripted {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{SHELL} {SCRIPT}")
    }
}
