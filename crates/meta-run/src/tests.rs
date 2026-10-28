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
const LEADER: &str = "leader\n";
const LIMIT: Duration = Duration::from_millis(60);
const PROMPT: Duration = Duration::from_secs(1);
const KILLED: [&str; 4] = [
    "sleep 5 & echo $! >> \"$0\"; echo $$ >> \"$0\"; echo held",
    "bash -c 'trap \"\" TERM; echo $$ >> \"$0\"; exec sleep 5' \"$0\" & echo $$ >> \"$0\"; wait",
    "trap '' TERM; echo $$ >> \"$0\"; for _ in 1 2 3 4 5; do sleep 1 & echo $! >> \"$0\"; wait; done",
    "echo $$ >> \"$0\"; kill -STOP $$",
];
const RECORDED: usize = 7;
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
    fn timed_out(script: &'static str, pids: &Path) -> String {
        let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, script]).arg(pids);
        let failure =
            run.ran(Instant::now() + LIMIT).err().filter(|failure| matches!(failure, RunFailure::TimedOut(_)));
        failure.map(|failure| failure.to_string()).unwrap_or_default()
    }

    fn gone(pids: &Path) -> bool {
        let listed: Vec<String> =
            std::fs::read_to_string(pids).unwrap_or_default().lines().map(str::to_owned).collect();
        let live = || {
            Run::new(PS[0], EnvPolicy::Inherit).args(&PS[1..]).arg(listed.join(",")).stdout(Instant::now() + PATIENT)
        };
        let until = Instant::now() + PROMPT;
        let polls = std::iter::repeat_with(|| std::thread::sleep(POLL)).take_while(|()| Instant::now() < until);
        let zombies = |states: String| states.lines().all(|state| state.starts_with('Z'));
        listed.len() >= RECORDED && polls.into_iter().any(|()| zombies(live().unwrap_or_default()))
    }
}

#[test]
fn a_run_feeds_its_input_returns_both_streams_and_leads_its_own_group() {
    let run = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, FED]).fed(SHOWN[0].to_owned());
    let ran = run.ran(Instant::now() + PATIENT).unwrap();
    assert!(!ran.success() && ran.stdout().text() == SHOWN[0] && ran.stderr().to_string() == SHOWN[1]);
    assert_eq!(ran.succeeded().unwrap_err().to_string(), SHOWN[2]);
    let leads = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, LEADS]);
    assert_eq!(leads.stdout(Instant::now() + PATIENT).unwrap(), LEADER);
}

#[test]
fn a_run_ends_at_its_deadline_and_kills_its_whole_group_holders_deaf_respawning_and_stalled_alike() {
    let killed = tempfile::NamedTempFile::new().unwrap();
    let started = Instant::now();
    let runs = KILLED.map(|script| (script, killed.path().to_path_buf()));
    let runs = runs.map(|(script, pids)| std::thread::spawn(move || Scripted::timed_out(script, &pids)));
    assert!(runs.into_iter().all(|run| run.join().unwrap().contains(STALLS)) && started.elapsed() < PROMPT);
    assert!(Scripted::gone(killed.path()), "{:?}", std::fs::read_to_string(killed.path()));
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
