use std::path::Path;

use meta_log::{GitLog, Snapshot};
use meta_severity::Scales;
use meta_state::State;
use task::{ChangeLink, Ledger, Policy, Standing};

use crate::task::ci::log_source::LogSource;
use crate::task::ci::run_kind::RunKind;
use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;

pub struct StandingLog;

impl StandingLog {
    const REMOTE: &str = "origin";
    const DEFAULT_BRANCH: &str = "origin/HEAD";
    const GIT_ENTRY: &str = ".git";

    pub fn git(root: &Path) -> GitLog {
        let Ok(remote) = Self::REMOTE.parse();
        let repository = root.ancestors().find(|dir| dir.join(Self::GIT_ENTRY).exists()).unwrap_or(root);
        GitLog::new(repository.to_path_buf(), remote)
    }

    pub fn prefetched(root: &Path) -> Result<LogSource, Failure> {
        Self::git(root).fetched_tip().map_err(Failure::GateFetch)?.map(LogSource::Prefetched).ok_or(Failure::NoLog)
    }

    pub fn state(snapshot: Result<Snapshot, Failure>) -> Result<State, Failure> {
        let snapshot = snapshot?;
        let records = snapshot.complete().map_err(Failure::StandingLog)?;
        Ok(State::fold(Some(records).filter(|records| !records.is_empty()).ok_or(Failure::NoLog)?))
    }

    pub fn counted(
        read: impl FnOnce() -> Result<Snapshot, Failure>,
        policy: &Policy,
        ledger: &mut Ledger,
        link: &ChangeLink,
    ) -> Result<(), Failure> {
        let standing =
            ledger.reads_standing(policy).then(|| Self::standing(read(), policy.scales(), link)).transpose()?;
        ledger.count(standing);
        Ok(())
    }

    pub fn enforcing(run: &RunKind, root: &Path, ledger: &mut Ledger) -> Result<(), Failure> {
        run.by_kind(
            ledger,
            |ledger| Self::enforced_from(root, ledger),
            |ledger| {
                ledger.report_only();
                Ok(())
            },
        )
    }

    pub fn enforced_from(root: &Path, ledger: &mut Ledger) -> Result<(), Failure> {
        let Ok(default_branch) = Self::DEFAULT_BRANCH.parse();
        let base = Revision::merge_base(root, &default_branch)?;
        let text = base.present(Path::new(PriorPolicy::PATH))?;
        ledger.enforce_from(text.as_deref()).map_err(Failure::PriorPolicy)
    }

    fn standing(snapshot: Result<Snapshot, Failure>, scales: &Scales, link: &ChangeLink) -> Result<Standing, Failure> {
        Ok(Standing::read(&Self::state(snapshot)?, link, scales))
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, LogStore, Snapshot};
    use settings::EmbeddedDefaults;
    use task::{ChangeLink, Ledger, Policy};

    use super::{Failure, LogSource, RunKind, StandingLog};
    use crate::task::ci::stranding::Stranding;
    use crate::task::fixture::Fixture;

    const CLOSED: &str = "standing budgets fail closed: meta/log";
    const GATE_CLOSED: &str = "local gate fails closed";
    const NOTHING: &str = "";
    const POLICY: &str = "tooling/policy.toml";
    const CAPS: &str = "\n[budgets.features-and-maintenance-critical]";
    const TOP: &str = "features-and-maintenance";
    const MINOR: &str = "minor";
    const UNREACHABLE: &str = "unreachable.git";
    const STANDING: &str = "\n[budgets.open-issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"issue\" } }\nallowance = { count = 20 }\nwindow = \"standing\"\n";

    impl StandingLog {
        fn policy(extra: &str) -> Policy {
            let bare = Policy::DEFAULTS.split(CAPS).next().unwrap_or_default();
            toml::from_str(&format!("{bare}{extra}")).expect("policy")
        }

        fn closed(refused: &[Result<(), Failure>], prefix: &str) -> bool {
            refused.iter().all(|refused| refused.as_ref().is_err_and(|failure| failure.to_string().starts_with(prefix)))
        }
    }

    #[test]
    fn the_merge_base_policy_decides_whether_standing_budgets_are_enforced() {
        let fixture = Fixture::new();
        fixture.write(POLICY, STANDING).branch();
        fixture.write(POLICY, NOTHING).committed();
        let (bare, mut pull, mut push) = (StandingLog::policy(NOTHING), Ledger::default(), Ledger::default());
        let enforced = StandingLog::enforcing(&RunKind::Pull(None, None), fixture.root(), &mut pull);
        let reported = StandingLog::enforcing(&RunKind::Other, fixture.root(), &mut push);
        assert!(enforced.is_ok() && pull.reads_standing(&bare) && !pull.reports_only());
        assert!(reported.is_ok() && push.reports_only());
    }

    #[test]
    fn an_unreadable_or_missing_log_fails_standing_budgets_closed_and_no_standing_budget_reads_nothing() {
        let reads: [fn() -> Result<Snapshot, Failure>; 2] =
            [|| Err(Failure::StandingLog(LogFailure::Contended)), || Ok(Snapshot::default())];
        let counted =
            |read, policy: &Policy| StandingLog::counted(read, policy, &mut Ledger::default(), &ChangeLink::default());
        assert!(counted(reads[0], &StandingLog::policy(NOTHING)).is_ok());
        let policy = StandingLog::policy(STANDING);
        let refused = reads.map(|read| counted(read, &policy));
        assert!(StandingLog::closed(&refused, CLOSED));
    }

    #[test]
    fn the_isolated_run_reads_exactly_the_fetched_tip_and_an_unreachable_or_unfetched_log_fails_closed() {
        let [writer, reader] = [Fixture::new(), Fixture::new()];
        writer.shared_origin(reader.write(UNREACHABLE, TOP));
        let append = || StandingLog::git(writer.root()).append::<LogFailure>(&mut |_| Ok(Stranding::rated(TOP, MINOR)));
        append().unwrap();
        let fetched = StandingLog::prefetched(reader.root()).unwrap();
        let log = StandingLog::git(reader.root());
        append().and_then(|_| log.read()).unwrap();
        reader.pointed(&reader.path(UNREACHABLE));
        let read = |source: &LogSource| source.snapshot(&log).map(|snapshot| snapshot.records().len());
        assert!(read(&fetched).ok() == Some(1) && read(&LogSource::Origin).is_err());
        let refused = [StandingLog::prefetched(reader.root()).map(drop), LogSource::Unfetched.snapshot(&log).map(drop)];
        assert!(StandingLog::closed(&refused, GATE_CLOSED));
    }
}
