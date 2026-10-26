mod gate_state;
mod posting;
mod status_argument;
mod unready;

pub use gate_state::GateState;
pub use posting::Posting;
pub use unready::Unready;

use std::path::{Path, PathBuf};

use task::{CommandLine, GateStatus, Policy, Program, Stdout};

use crate::task::ci::step_times::StepTimes;
use crate::task::failure::Failure;
use crate::task::revision::Commit;
use status_argument::StatusArgument;

#[derive(Debug)]
pub struct LocalGate {
    root: PathBuf,
    gh: CommandLine,
    status: GateStatus,
    source: PathBuf,
}

impl LocalGate {
    const SOURCE: &str = env!("CARGO_MANIFEST_DIR");
    const CHANGES: [&str; 2] = ["status", "--porcelain"];
    const PUSHED: [&str; 4] = ["branch", "--remotes", "--contains", "HEAD"];
    const HEAD: [&str; 2] = ["rev-parse", "HEAD"];
    const VERSION: [&str; 3] = ["rev-parse", "--short", "HEAD"];
    const POST: [&str; 4] = ["api", "--method", "POST", "--silent"];
    const FIELD: &str = "-f";

    pub fn new(root: &Path, policy: &Policy) -> Result<LocalGate, Failure> {
        Ok(LocalGate {
            root: root.to_path_buf(),
            gh: CommandLine::new(Program::GitHub),
            status: policy.gate()?.clone(),
            source: PathBuf::from(Self::SOURCE),
        })
    }

    pub fn ready(&self) -> Result<Commit, Failure> {
        let git = |arguments: &[&str]| Self::git(&self.root).arguments(arguments).captured::<Stdout>();
        match (git(&Self::CHANGES)?.is_empty(), git(&Self::PUSHED)?.is_empty()) {
            (false, _) => Err(Unready::Dirty.into()),
            (true, true) => Err(Unready::Unpushed.into()),
            (true, false) => git(&Self::HEAD)?.parse(),
        }
    }

    pub fn post(&self, checked: &Commit, times: &StepTimes) -> Result<(), Failure> {
        (self.ready()? == *checked).then_some(()).ok_or(Unready::Moved)?;
        let version = Self::git(&self.source).arguments(Self::VERSION).captured::<Stdout>()?.parse()?;
        let state = times.state();
        let line = self.gh.clone().in_directory(self.root.clone()).arguments(Self::POST);
        [
            StatusArgument::State(state),
            StatusArgument::Context(&self.status),
            StatusArgument::Description { version, times },
        ]
        .iter()
        .fold(line.argument(StatusArgument::Endpoint(checked)), |line, field| {
            line.argument(Self::FIELD).argument(field)
        })
        .run()?;
        println!("{}: posted {state} on {checked}", self.status);
        Ok(())
    }

    fn git(directory: &Path) -> CommandLine {
        CommandLine::new(Program::Git).in_directory(directory.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use settings::EmbeddedDefaults;
    use task::{CommandLine, Policy, Program};

    use super::{LocalGate, Unready};
    use crate::task::ci::ci_step::CiStep;
    use crate::task::ci::step_times::StepTimes;
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;

    const SOURCE: &str = "src/lib.rs";
    const ALPHA: &str = "pub struct Alpha;\n";
    const BETA: &str = "pub struct Beta;\n";
    const SHELL: &str = "sh";
    const FAKE_GH: &str = ".git/fake-gh";
    const RECORDED: &str = ".git/fake-gh.args";
    const SCRIPT: &str = "printf '%s\\n' \"$@\" > \"$0.args\"\n";
    const SECOND: Duration = Duration::from_secs(1);
    const POSTED: &str = "-f\nstate=success\n-f\ncontext=local-gate\n-f\ndescription=meta-dev ";
    const TIMED: &str = ": fmt 1.0s\n";

    impl LocalGate {
        fn faked(fixture: &Fixture) -> LocalGate {
            fixture.write(SOURCE, ALPHA).branch();
            fs::write(fixture.path(FAKE_GH), SCRIPT).expect("write fake gh");
            let gh = CommandLine::new(Program::Executable(SHELL.into())).argument(fixture.path(FAKE_GH).display());
            let gate = LocalGate::new(fixture.root(), &Policy::defaults()).expect("gate in policy");
            LocalGate { gh, source: fixture.root().into(), ..gate }
        }

        fn refused(outcome: Result<impl Sized, Failure>, unready: Unready) -> bool {
            outcome.err().map(|failure| failure.to_string()) == Some(Failure::from(unready).to_string())
        }
    }

    #[test]
    fn only_a_clean_tree_at_a_pushed_head_is_ready() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        assert!(gate.ready().is_ok());
        fixture.write(SOURCE, BETA);
        assert!(LocalGate::refused(gate.ready(), Unready::Dirty));
        fixture.commit();
        assert!(LocalGate::refused(gate.ready(), Unready::Unpushed));
    }

    #[test]
    fn the_checked_commit_gets_the_state_the_status_name_the_version_and_the_timings() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let checked = gate.ready().unwrap();
        let mut times = StepTimes::default();
        times.record(CiStep::Fmt, Ok(SECOND)).unwrap();
        gate.post(&checked, &times).unwrap();
        let recorded = fs::read_to_string(fixture.path(RECORDED)).unwrap_or_default();
        assert!(recorded.contains(&format!("/statuses/{checked}\n{POSTED}")) && recorded.ends_with(TIMED));
    }

    #[test]
    fn a_tree_dirtied_or_a_head_moved_during_the_run_posts_nothing() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let checked = gate.ready().unwrap();
        fixture.write(SOURCE, BETA);
        assert!(LocalGate::refused(gate.post(&checked, &StepTimes::default()), Unready::Dirty));
        fixture.branch();
        let moved = LocalGate::refused(gate.post(&checked, &StepTimes::default()), Unready::Moved);
        assert!(moved && !fixture.path(RECORDED).exists());
    }
}
