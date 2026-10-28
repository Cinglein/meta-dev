use std::os::unix::process::CommandExt as _;
use std::time::{Duration, Instant};

use super::Run;
use super::group::Group;
use crate::ran::Ran;
use crate::run_failure::RunFailure;

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
                run.finished(deadline, |_| Some(()), Run::alone)
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
    use crate::env_policy::EnvPolicy;
    use crate::run::Run;
    use std::time::{Duration, Instant};

    const SHELL: &str = "bash";
    const SCRIPT: &str = "-c";
    const LEADS: &str = "kill -0 -- -$$ 2>/dev/null && echo leader; true";
    const PTY: &str = "script -qec \"$0\" /dev/null 2>/dev/null || script -q /dev/null sh -c \"$0\"";
    const CHILD: &str = "--exact run::placement::tests::under_a_terminal_it_shares_the_callers_group --ignored";
    const LEADER: &str = "leader\n";
    const TERMINAL: &str = "/dev/tty";
    const PASSED: &str = "1 passed";
    const PATIENT: Duration = Duration::from_secs(5);

    #[test]
    #[ignore = "run under a pseudo-terminal by the test below"]
    fn under_a_terminal_it_shares_the_callers_group() {
        let leads = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, LEADS]).terminal();
        assert_eq!(leads.stdout(Instant::now() + PATIENT).unwrap(), "");
    }

    #[test]
    fn a_terminal_run_leads_its_own_group_unless_meta_has_a_controlling_terminal() {
        let leads =
            Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, LEADS]).terminal().stdout(Instant::now() + PATIENT);
        let child = format!("{} {CHILD}", std::env::current_exe().unwrap().display());
        let pty = Run::new(SHELL, EnvPolicy::Inherit).args([SCRIPT, PTY, &child]).stdout(Instant::now() + PATIENT);
        assert!(leads.is_ok_and(|leads| leads == std::fs::File::open(TERMINAL).map_or(LEADER, |_| "")));
        assert!(pty.is_ok_and(|shown| shown.contains(PASSED)));
    }
}
