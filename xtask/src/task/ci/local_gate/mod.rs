mod busy;
mod checkout;
mod config_effect;
mod findings;
mod gate_child;
mod gate_lock;
mod gate_state;
mod meta_rev;
mod outside_config;
mod passed_steps;
mod posting;
mod recorded_times;
mod status_argument;
mod status_description;
mod toolchain;
mod unready;
mod worktrees;

pub use busy::Busy;
pub use outside_config::OutsideConfig;
pub use posting::Posting;
pub use unready::Unready;

use checkout::Checkout;
use config_effect::ConfigEffect;
use findings::Findings;
use gate_child::GateChild;
use gate_lock::GateLock;
use gate_state::GateState;
use meta_rev::MetaRev;
use passed_steps::PassedSteps;
use recorded_times::RecordedTimes;
use status_description::StatusDescription;
use toolchain::Toolchain;
use worktrees::Worktrees;

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use meta_schema::PullRequest;

use task::{Branch, CommandLine, GateEnvName, GateStatus, Policy, Program, Stdout};

use crate::task::cargo_metadata::CargoMetadata;
use crate::task::ci::head_branch::HeadBranch;
use crate::task::ci::run_kind::RunKind;
use crate::task::ci::standing_log::StandingLog;
use crate::task::failure::Failure;
use crate::task::revision::Commit;
use crate::task::surfaces::Surfaces;
use status_argument::StatusArgument;

#[derive(Debug)]
pub struct LocalGate {
    root: PathBuf,
    gh: CommandLine,
    status: GateStatus,
    passed: Vec<GateEnvName>,
}

impl LocalGate {
    pub const TIMES: &str = "META_GATE_TIMES";
    const CHANGES: [&str; 3] = ["status", "--porcelain", "--untracked-files=all"];
    const PUSHED: [&str; 4] = ["branch", "--remotes", "--contains", "HEAD"];
    const HEAD: [&str; 2] = ["rev-parse", "HEAD"];
    const TIPS: [&str; 4] = ["branch", "--remotes", "--format=%(refname:lstrip=3)", "--points-at"];
    const REMOTE_HEAD: &str = "HEAD";
    const POST: [&str; 4] = ["api", "--method", "POST", "--silent"];
    const PULLS: [&str; 9] = ["pr", "list", "--state", "open", "--json", "number", "--jq", ".[].number", "--head"];
    const GATE: [&str; 2] = ["xtask", "ci"];
    const FIELD: &str = "-f";
    const CHECKOUTS: &str = "meta-local-gate";
    const CARGO: &str = "cargo";
    const METADATA: [&str; 4] = ["metadata", "--format-version", "1", "--all-features"];
    const WORKTREES: [&str; 3] = ["worktree", "list", "--porcelain"];

    pub fn new(root: &Path, policy: &Policy) -> Result<LocalGate, Failure> {
        Ok(LocalGate {
            root: root.to_path_buf(),
            gh: CommandLine::new(Program::GitHub),
            status: policy.gate()?.clone(),
            passed: policy.gate_env().to_vec(),
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
        let listed: Worktrees = Self::git(&self.root).arguments(Self::WORKTREES).captured::<Stdout>()?.parse()?;
        let key = listed.main().unwrap_or(&self.root).join(Checkout::scope_of(&self.root));
        let target = GateLock::target(&parent, &key);
        let lock = GateLock::guarding(&target, policy.gate_wait()?)?;
        let checkout = self.checkout(&parent)?;
        let head = self.head_branch(checkout.commit())?;
        let number = self.open_pull(&head)?.as_ref().map(ToString::to_string).unwrap_or_default();
        let ambient = CommandLine::ambient().into_iter().chain([(RunKind::NUMBER.to_owned(), number.into())]).collect();
        let log = StandingLog::prefetched(&checkout.root())?;
        let toolchain = checkout.toolchain()?;
        let child = GateChild::new(&parent, target, checkout.times().path().to_path_buf(), toolchain.clone())?
            .seeing(ambient)
            .passing(&self.passed)
            .heading(Some(head));
        let cargo = || child.line(Program::Executable(PathBuf::from(Self::CARGO)), checkout.root());
        let metadata: CargoMetadata = cargo().arguments(Self::METADATA).captured::<Stdout>()?.parse()?;
        OutsideConfig::around(checkout.above().unwrap_or(&parent), child.cargo_home())
            .and(metadata.outside(|manifest| checkout.holds(manifest)))
            .refused()?;
        let outcome = cargo().arguments(Self::GATE).argument(log).run_holding(lock.inheritable()?);
        let changed = Surfaces::changed(&checkout.root(), policy.surfaces(), policy.branch(Branch::Base))?;
        changed.announce(summary)?;
        self.concluded(&checkout, outcome, &Findings::new(changed, toolchain))
    }

    pub fn concluded(
        &self,
        checkout: &Checkout,
        outcome: Result<(), task::Failure>,
        findings: &Findings,
    ) -> Result<(), Failure> {
        let recorded = outcome.map_err(Failure::from).and_then(|()| {
            checkout.times().passed()?.whole().then_some(()).ok_or(Failure::Unready(Unready::Incomplete))
        });
        let state = recorded.as_ref().map_or(GateState::Failure, |()| GateState::Success);
        self.post(checkout, state, findings).and(recorded)
    }

    fn post(&self, checkout: &Checkout, state: GateState, findings: &Findings) -> Result<(), Failure> {
        let checked = checkout.commit();
        checkout.intact()?;
        let version = checkout.version()?;
        let times = checkout.times();
        let context = StatusArgument::Context { status: &self.status, scope: checkout.scope() };
        let line = self.gh.clone().in_directory(self.root.clone()).arguments(Self::POST);
        let description = StatusArgument::Description {
            version,
            state,
            toolchain: findings.toolchain(),
            times: &times,
            surfaces: findings.surfaces(),
            passed: &self.passed,
        };
        [StatusArgument::State(state), context, description]
            .iter()
            .fold(line.argument(StatusArgument::Endpoint(checked)), |line, field| {
                line.argument(Self::FIELD).argument(field)
            })
            .run()?;
        println!("{}: posted {state} on {checked}", self.status);
        Ok(())
    }

    pub fn open_pull(&self, head: &HeadBranch) -> Result<Option<PullRequest>, Failure> {
        let line = self.gh.clone().in_directory(self.root.clone()).arguments(Self::PULLS).argument(head);
        let listed = line.captured::<Stdout>()?;
        let open: Vec<&str> = listed.lines().collect();
        Ok(match open.as_slice() {
            [only] => only.parse().ok(),
            _ => None,
        })
    }

    pub fn head_branch(&self, commit: &Commit) -> Result<HeadBranch, Failure> {
        let listed = Self::git(&self.root).arguments(Self::TIPS).argument(commit).captured::<Stdout>()?;
        let names: BTreeSet<&str> = listed.lines().filter(|name| *name != Self::REMOTE_HEAD).collect();
        let only = names.first().filter(|_| names.len() == 1).ok_or(Unready::Unnamed)?;
        let Ok(head) = only.parse();
        Ok(head)
    }

    fn git(directory: &Path) -> CommandLine {
        CommandLine::new(Program::Git).in_directory(directory.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use meta_door::Door;
    use meta_door::kinds::Kind;
    use std::path::PathBuf;

    use settings::EmbeddedDefaults;
    use strum::IntoEnumIterator;
    use task::{CommandLine, Policy, Program};

    use super::{Findings, LocalGate, Unready};
    use crate::task::ci::CiStep;
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;
    use crate::task::surfaces::Changed;

    const IGNORE: &str = ".gitignore";
    const IGNORED: &str = "target/\n*.built\n";
    const LEFTOVER: &str = "leftover.built";
    const LOCKFILE: &str = "Cargo.lock";
    const SOURCE: &str = "consumer/src/lib.rs";
    const SCOPE: &str = "consumer";
    const ALPHA: &str = "pub struct Alpha;\n";
    const BETA: &str = "pub struct Beta;\n";
    const SHELL: &str = "sh";
    const FAKE_GH: &str = "../fake-gh";
    const RECORDED: &str = "../fake-gh.args";
    const SCRIPT: &str = "printf '%s\\n' \"$@\" > \"$0.args\"\n";
    const TIMED: &str = "fmt 1.0s";
    const POSTED: &str = "-f\nstate=success\n-f\ncontext=local-gate/consumer\n-f\ndescription=meta-dev local on 1.90-x86_64-unknown-linux-gnu: ";
    const TOOLCHAIN: &str = "1.90-x86_64-unknown-linux-gnu";
    const ALL_FEATURES: &str = "--all-features";
    const FAILED: &str = "state=failure";
    const SURFACED: &str = "; surfaces changed\n";
    const TIMED_ALL: &str = ": fmt 1.0s, clippy 1.0s,";
    const PUSHED_AT_HEAD: &str = "main";
    const SECOND_PUSH: &str = "refs/remotes/origin/second";
    const LISTING: &str = "printf '7\\n'\n";
    const LISTED: &str = "7";
    const HEAD: &str = "HEAD";

    impl CiStep {
        fn every_step_passed() -> String {
            CiStep::iter().map(|step| format!("{step} 1.0s")).collect::<Vec<String>>().join(", ")
        }
    }

    impl LocalGate {
        fn faked(fixture: &Fixture) -> LocalGate {
            fixture.write(IGNORE, IGNORED).write(SOURCE, ALPHA).branch();
            fixture.write(FAKE_GH, SCRIPT);
            let gh = CommandLine::new(Program::Executable(SHELL.into())).argument(fixture.path(FAKE_GH).display());
            let gate = LocalGate::new(&fixture.path(SCOPE), &Policy::defaults()).expect("gate in policy");
            LocalGate { gh, ..gate }
        }

        fn refused(outcome: Result<impl Sized, Failure>, unready: Unready) -> bool {
            outcome.err().map(|failure| failure.to_string()) == Some(Failure::from(unready).to_string())
        }
    }

    #[test]
    fn only_a_clean_tree_at_a_pushed_head_is_ready_and_the_judged_branch_is_the_one_pushed_at_head() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let named = || gate.head_branch(&fixture.revision(HEAD).parse().unwrap());
        let pushed = named().ok() == PUSHED_AT_HEAD.parse().ok();
        assert!(pushed && gate.ready().is_ok() && LocalGate::METADATA.contains(&ALL_FEATURES));
        fixture.updated(SECOND_PUSH);
        let several = LocalGate::refused(named(), Unready::Unnamed);
        fixture.write(FAKE_GH, LISTING);
        let listed = gate.open_pull(&PUSHED_AT_HEAD.parse().unwrap()).unwrap() == LISTED.parse().ok();
        fixture.write(SOURCE, BETA);
        assert!(several && listed && LocalGate::refused(gate.ready(), Unready::Dirty));
        fixture.committed();
        assert!(LocalGate::refused(gate.ready(), Unready::Unpushed) && LocalGate::refused(named(), Unready::Unnamed));
    }

    #[test]
    fn the_checked_commit_gets_the_childs_state_a_scoped_context_the_version_timings_and_surfaces() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let checkout = gate.checkout(&fixture.beside()).unwrap();
        Door::write(checkout.times().path(), CiStep::every_step_passed()).unwrap();
        Door::write(checkout.root().join(LOCKFILE), BETA).unwrap();
        fixture.write(SOURCE, BETA).branch();
        let surfaces = Findings::new(Changed::new(vec![PathBuf::from(IGNORE)]), TOOLCHAIN.parse().unwrap());
        let recorded = || Door::read(fixture.path(RECORDED)).unwrap_or_default();
        gate.concluded(&checkout, Ok(()), &surfaces).unwrap();
        let posted = recorded();
        let described = posted.contains(TIMED_ALL) && posted.ends_with(SURFACED);
        assert!(posted.contains(&format!("/statuses/{}\n{POSTED}", checkout.commit())) && described);
        let failed = gate.concluded(&checkout, Err(task::Failure::NoWorkspace), &surfaces);
        assert!(failed.is_err() && recorded().contains(FAILED));
        Door::write(checkout.times().path(), TIMED).unwrap();
        let partial = gate.concluded(&checkout, Ok(()), &surfaces);
        assert!(LocalGate::refused(partial, Unready::Incomplete) && recorded().contains(FAILED));
    }

    #[test]
    fn a_changed_checkout_posts_nothing() {
        let fixture = Fixture::new();
        let gate = LocalGate::faked(&fixture);
        let checkout = gate.checkout(&fixture.beside()).unwrap();
        Door::write(checkout.root().join(LEFTOVER), BETA).unwrap();
        let findings = Findings::new(Changed::new(Vec::new()), TOOLCHAIN.parse().unwrap());
        let posted = gate.concluded(&checkout, Ok(()), &findings);
        assert!(LocalGate::refused(posted, Unready::Tampered) && Door::kind(fixture.path(RECORDED)) == Kind::Missing);
    }
}
