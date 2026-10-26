mod busy;
mod checkout;
mod gate_lock;
mod gate_state;
mod meta_rev;
mod posting;
mod recorded_times;
mod status_argument;
mod unready;
mod worktrees;

pub use busy::Busy;
pub use posting::Posting;
pub use unready::Unready;

use checkout::Checkout;
use gate_lock::GateLock;
use gate_state::GateState;
use meta_rev::MetaRev;
use recorded_times::RecordedTimes;
use worktrees::Worktrees;

use std::env;
use std::path::{Path, PathBuf};

use task::{Branch, CommandLine, GateStatus, Policy, Program, Stdout};

use crate::task::failure::Failure;
use crate::task::revision::Commit;
use crate::task::surfaces::{Changed, Surfaces};
use status_argument::StatusArgument;

#[derive(Debug)]
pub struct LocalGate {
    root: PathBuf,
    gh: CommandLine,
    status: GateStatus,
}

impl LocalGate {
    pub const TIMES: &str = "META_GATE_TIMES";
    const CHANGES: [&str; 3] = ["status", "--porcelain", "--untracked-files=all"];
    const PUSHED: [&str; 4] = ["branch", "--remotes", "--contains", "HEAD"];
    const HEAD: [&str; 2] = ["rev-parse", "HEAD"];
    const POST: [&str; 4] = ["api", "--method", "POST", "--silent"];
    const GATE: [&str; 2] = ["xtask", "ci"];
    const FIELD: &str = "-f";
    const CHECKOUTS: &str = "meta-local-gate";
    const CARGO_TARGET: &str = "CARGO_TARGET_DIR";

    pub fn new(root: &Path, policy: &Policy) -> Result<LocalGate, Failure> {
        Ok(LocalGate {
            root: root.to_path_buf(),
            gh: CommandLine::new(Program::GitHub),
            status: policy.gate()?.clone(),
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

    pub fn checkout(&self, parent: &Path) -> Result<Checkout, Failure> {
        let parent = Checkout::sweep(&self.root, parent)?;
        Checkout::add(&self.root, self.ready()?, &parent)
    }

    pub fn run(&self, policy: &Policy, summary: Option<&Path>) -> Result<(), Failure> {
        let parent = env::temp_dir().join(Self::CHECKOUTS);
        let target = GateLock::target(&parent, &self.root);
        let _lock = GateLock::guarding(&target, policy.gate_wait()?)?;
        let checkout = self.checkout(&parent)?;
        let outcome = CommandLine::new(Program::Cargo)
            .in_directory(checkout.root())
            .arguments(Self::GATE)
            .environment(Self::CARGO_TARGET, target)
            .environment(Self::TIMES, checkout.times().path().to_path_buf())
            .run();
        let changed = Surfaces::changed(&checkout.root(), policy.surfaces(), policy.branch(Branch::Base))?;
        changed.announce(summary)?;
        self.concluded(&checkout, outcome, &changed)
    }

    pub fn concluded(
        &self,
        checkout: &Checkout,
        outcome: Result<(), task::Failure>,
        surfaces: &Changed,
    ) -> Result<(), Failure> {
        let state = outcome.as_ref().map_or(GateState::Success, |()| GateState::Failure);
        self.post(checkout, state, surfaces).and(outcome.map_err(Failure::from))
    }

    fn post(&self, checkout: &Checkout, state: GateState, surfaces: &Changed) -> Result<(), Failure> {
        let checked = checkout.commit();
        checkout.intact()?;
        let version = checkout.version()?;
        let times = checkout.times();
        let context = StatusArgument::Context { status: &self.status, scope: checkout.scope() };
        let line = self.gh.clone().in_directory(self.root.clone()).arguments(Self::POST);
        [StatusArgument::State(state), context, StatusArgument::Description { version, times: &times, surfaces }]
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
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;
    use task::{CommandLine, Policy, Program};

    use super::{LocalGate, Unready};
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;
    use crate::task::surfaces::Changed;

    const IGNORE: &str = ".gitignore";
    const IGNORED: &str = "target/\n";
    const SOURCE: &str = "consumer/src/lib.rs";
    const SCOPE: &str = "consumer";
    const ALPHA: &str = "pub struct Alpha;\n";
    const BETA: &str = "pub struct Beta;\n";
    const SHELL: &str = "sh";
    const FAKE_GH: &str = ".git/fake-gh";
    const RECORDED: &str = ".git/fake-gh.args";
    const SCRIPT: &str = "printf '%s\\n' \"$@\" > \"$0.args\"\n";
    const TIMED: &str = "fmt 1.0s";
    const POSTED: &str = "-f\nstate=success\n-f\ncontext=local-gate/consumer\n-f\ndescription=meta-dev local: fmt 1.0s; surfaces changed\n";
    const FAILED: &str = "state=failure";

    impl LocalGate {
        fn faked(fixture: &Fixture) -> LocalGate {
            fixture.write(IGNORE, IGNORED).write(SOURCE, ALPHA).branch();
            fs::write(fixture.path(FAKE_GH), SCRIPT).expect("write fake gh");
            let gh = CommandLine::new(Program::Executable(SHELL.into())).argument(fixture.path(FAKE_GH).display());
            let gate = LocalGate::new(&fixture.path(SCOPE), &Policy::defaults()).expect("gate in policy");
            LocalGate { gh, ..gate }
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
        fixture.committed();
        assert!(LocalGate::refused(gate.ready(), Unready::Unpushed));
    }

    #[test]
    fn the_checked_commit_gets_the_childs_state_a_scoped_context_the_version_timings_and_surfaces() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let checkout = gate.checkout(&fixture.beside()).unwrap();
        fs::write(checkout.times().path(), TIMED).unwrap();
        fixture.write(SOURCE, BETA).branch();
        let surfaces = Changed::new(vec![PathBuf::from(IGNORE)]);
        gate.concluded(&checkout, Ok(()), &surfaces).unwrap();
        let recorded = fs::read_to_string(fixture.path(RECORDED)).unwrap_or_default();
        assert!(recorded.ends_with(&format!("/statuses/{}\n{POSTED}", checkout.commit())));
        let failed = gate.concluded(&checkout, Err(task::Failure::NoWorkspace), &surfaces);
        assert!(failed.is_err() && fs::read_to_string(fixture.path(RECORDED)).unwrap_or_default().contains(FAILED));
    }

    #[test]
    fn a_changed_checkout_posts_nothing() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let checkout = gate.checkout(&fixture.beside()).unwrap();
        fs::write(checkout.root().join(SCOPE), BETA).unwrap();
        let posted = gate.concluded(&checkout, Ok(()), &Changed::new(Vec::new()));
        assert!(LocalGate::refused(posted, Unready::Tampered) && !fixture.path(RECORDED).exists());
    }
}
