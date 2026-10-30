mod process_id;

use std::os::unix::process::CommandExt as _;
use std::time::{Duration, Instant};

use super::Run;
use super::group::Group;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use process_id::ProcessId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Own,
    Nested,
    Terminal,
}

impl Placement {
    const TERMINAL: &'static str = "/dev/tty";

    pub fn ran(self, run: Run, deadline: Instant) -> Result<Ran, RunFailure> {
        match self {
            Self::Terminal if std::fs::File::open(Self::TERMINAL).is_ok() => {
                run.finished(deadline, |_| Some(()), ProcessId::ended)
            }
            Self::Nested => Self::grouped(run, deadline, Group::SETTLE),
            Self::Own | Self::Terminal => Self::grouped(run, deadline, Group::HANDOFF),
        }
    }

    fn grouped(mut run: Run, deadline: Instant, patience: Duration) -> Result<Ran, RunFailure> {
        run.command.process_group(i32::default());
        run.finished(deadline, |leader| Group::joined(leader, patience), |leader| Group::ended(leader, patience))
    }
}

#[cfg(test)]
mod tests {
    use super::super::group::Group;
    use super::Placement;
    use crate::env_policy::EnvPolicy;
    use crate::run::Run;
    use crate::run_failure::RunFailure;
    use nix::sys::signal::kill;
    use nix::unistd::Pid;
    use std::time::{Duration, Instant};

    const BASH: &str = "bash";
    const SCRIPT: &str = "-c";
    const LEADS: &str = "kill -0 -- -$$ 2>/dev/null && echo leader; echo -$$";
    const ALONE: &str = "echo $$ > \"$0\"; exec sleep 5";
    const ALIVE: &str = "kill -0 $0 2>/dev/null && echo alive; true";
    const PTY: &str = "script -qec \"$0\" /dev/null 2>/dev/null || script -q /dev/null sh -c \"$0\"";
    const CHILD: &str =
        "--exact run::placement::tests::in_a_terminal_it_shares_the_group_and_closing_refuses --ignored";
    const HELPER: &str = "bash -c 'sleep 5 & echo $! > \"$0\"; wait' \"$0\" & wait";
    const POLL: Duration = Duration::from_millis(2);
    const WINDOW: Duration = Duration::from_secs(1);
    const LEADER: &str = "leader";
    const PROMPT: &str = "GIT_TERMINAL_PROMPT";
    const PASSED: &str = "1 passed";
    const PATIENT: Duration = Duration::from_secs(5);
    const LIMIT: Duration = Duration::from_millis(60);
    const MARGIN: Duration = LIMIT.saturating_mul(5);

    #[test]
    #[ignore = "run under a pseudo-terminal by the test below"]
    fn in_a_terminal_it_shares_the_group_and_closing_refuses() {
        let shell = |script: &str| Run::new(BASH, EnvPolicy::Inherit).args([SCRIPT, script]);
        let leads = shell(LEADS).terminal().stdout(Instant::now() + PATIENT).unwrap_or_default();
        let helper = tempfile::NamedTempFile::new().unwrap();
        let expired = shell(HELPER).arg(helper.path()).terminal().ran(Instant::now() + MARGIN);
        let pid = std::fs::read_to_string(helper.path()).unwrap_or_default().trim().parse().ok().map(Pid::from_raw);
        let held = || pid.is_none_or(|pid| kill(pid, None).is_ok());
        let until = Instant::now() + WINDOW;
        let polls = std::iter::repeat_with(|| std::thread::sleep(POLL)).take_while(|()| Instant::now() < until);
        let ended = matches!(expired, Err(RunFailure::TimedOut(_))) && polls.into_iter().any(|()| !held());
        Group::all_ended();
        let refused = shell(LEADS).ran(Instant::now() + PATIENT);
        assert!(!leads.is_empty() && !leads.contains(LEADER) && matches!(refused, Err(RunFailure::Interrupted(_))));
        assert!(ended && !std::fs::read_to_string(helper.path()).unwrap_or_default().is_empty());
    }

    #[test]
    fn terminal_runs_lead_their_group_without_a_terminal_let_git_prompt_and_runs_and_helpers_leave_nothing_behind() {
        let recorded = tempfile::NamedTempFile::new().unwrap();
        let shell = |script: &str| Run::new(BASH, EnvPolicy::Inherit).args([SCRIPT, script]);
        let helped = shell(ALONE).arg(recorded.path()).helped(LIMIT);
        let pid = std::fs::read_to_string(recorded.path()).unwrap_or_default();
        let alive = shell(ALIVE).arg(pid.trim()).stdout(Instant::now() + PATIENT).unwrap_or_default();
        let shown = shell(LEADS).terminal().stdout(Instant::now() + PATIENT).unwrap_or_default();
        let group = shown.lines().last().unwrap_or_default();
        let child = format!("{} {CHILD}", std::env::current_exe().unwrap().display());
        let pty = shell(PTY).arg(child).stdout(Instant::now() + PATIENT).unwrap_or_default();
        let detached = std::fs::File::open(Placement::TERMINAL).map_or(LEADER, |_| group);
        assert!(shown.starts_with(detached) && !Group::live().keys().any(|live| live.to_string() == group));
        assert!(
            matches!(helped, Err(RunFailure::TimedOut(_)))
                && alive.is_empty()
                && !pid.is_empty()
                && pty.contains(PASSED)
        );
        let terminal = Run::git(BASH.as_ref()).terminal();
        assert!(terminal.command.get_envs().all(|(name, value)| name != PROMPT || value.is_none()));
    }
}
