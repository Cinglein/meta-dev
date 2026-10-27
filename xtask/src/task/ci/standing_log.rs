use std::env;
use std::path::Path;

use meta_log::{GitLog, Snapshot};
use meta_severity::Scales;
use meta_state::State;
use task::{ChangeLink, Ledger, Policy, Standing};

use crate::task::ci::log_source::LogSource;
use crate::task::ci::stranding::Stranding;
use crate::task::failure::Failure;

pub struct StandingLog;

impl StandingLog {
    const REMOTE: &str = "origin";
    const GIT_ENTRY: &str = ".git";
    const PULL_REQUEST: &str = "PULL_REQUEST";
    const HEAD: &str = "GITHUB_HEAD_REF";

    pub fn git(root: &Path) -> GitLog {
        let Ok(remote) = Self::REMOTE.parse();
        let repository = root.ancestors().find(|dir| dir.join(Self::GIT_ENTRY).exists()).unwrap_or(root);
        GitLog::new(repository.to_path_buf(), remote)
    }

    pub fn prefetched(root: &Path, policy: &Policy) -> Result<LogSource, Failure> {
        if policy.budgets().standing() || Stranding::rescaling(root, policy)?.is_some() {
            Self::git(root).fetched_tip().map_err(Failure::GateFetch)?.map(LogSource::Prefetched).ok_or(Failure::NoLog)
        } else {
            Ok(LogSource::Unfetched)
        }
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
    ) -> Result<(), Failure> {
        let standing = policy.budgets().standing().then(|| Self::standing(read(), policy.scales())).transpose()?;
        ledger.count(standing);
        Ok(())
    }

    fn standing(snapshot: Result<Snapshot, Failure>, scales: &Scales) -> Result<Standing, Failure> {
        let state = Self::state(snapshot)?;
        let link = ChangeLink::new(
            env::var(Self::PULL_REQUEST).ok().and_then(|number| number.parse().ok()),
            env::var(Self::HEAD).ok().and_then(|branch| branch.parse().ok()),
        );
        Ok(Standing::read(&state, &link, scales))
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, LogStore, Snapshot};
    use settings::EmbeddedDefaults;
    use task::{Ledger, Policy};

    use super::{Failure, LogSource, StandingLog, Stranding};
    use crate::task::fixture::Fixture;

    const CLOSED: &str = "standing budgets fail closed: meta/log";
    const GATE_CLOSED: &str = "local gate fails closed";
    const TOP: &str = "features-and-maintenance";
    const MINOR: &str = "minor";
    const UNREACHABLE: &str = "unreachable.git";
    const STANDING: &str = "\n[budgets.open-issues]\ndivision = \"features-and-maintenance\"\nmeasure = { open_items = { kind = \"issue\" } }\nallowance = { count = 20 }\nwindow = \"standing\"\n";
    const SCALED: &str = "\n[[scales.features-and-maintenance]]\nname = \"blocker\"\nmeaning = \"x\"\n\n[[scales.features-and-maintenance]]\nname = \"nuisance\"\nmeaning = \"x\"\n";

    impl StandingLog {
        fn policy(extra: &str) -> Policy {
            toml::from_str(&format!("{}{extra}", Policy::DEFAULTS)).expect("policy")
        }

        fn closed(refused: &[Result<(), Failure>], prefix: &str) -> bool {
            refused.iter().all(|refused| refused.as_ref().is_err_and(|failure| failure.to_string().starts_with(prefix)))
        }
    }

    #[test]
    fn an_unreadable_or_missing_log_fails_standing_budgets_closed_and_no_standing_budget_reads_nothing() {
        let reads: [fn() -> Result<Snapshot, Failure>; 2] =
            [|| Err(Failure::StandingLog(LogFailure::Contended)), || Ok(Snapshot::default())];
        assert!(StandingLog::counted(reads[0], &Policy::defaults(), &mut Ledger::default()).is_ok());
        let policy = StandingLog::policy(STANDING);
        let refused = reads.map(|read| StandingLog::counted(read, &policy, &mut Ledger::default()));
        assert!(StandingLog::closed(&refused, CLOSED));
    }

    #[test]
    fn the_isolated_run_reads_exactly_the_fetched_tip_and_an_unfetched_run_fails_closed_despite_a_tracking_ref() {
        let [writer, reader] = [Fixture::new(), Fixture::new()];
        writer.shared_origin(reader.write(UNREACHABLE, TOP));
        reader.branch();
        let append = || StandingLog::git(writer.root()).append::<LogFailure>(&mut |_| Ok(Stranding::rated(TOP, MINOR)));
        append().unwrap();
        let [standing, scaled] = [STANDING, SCALED].map(StandingLog::policy);
        let [fetched, skipped] =
            [&standing, &Policy::defaults()].map(|policy| StandingLog::prefetched(reader.root(), policy).unwrap());
        let log = StandingLog::git(reader.root());
        append().and_then(|_| log.read()).unwrap();
        reader.pointed(&reader.path(UNREACHABLE));
        let read = |source: &LogSource| source.snapshot(&log).map(|snapshot| snapshot.records().len());
        assert!(read(&fetched).ok() == Some(1) && read(&LogSource::Origin).is_err());
        let counted = StandingLog::counted(|| skipped.snapshot(&log), &standing, &mut Ledger::default());
        let refused = [&standing, &scaled].map(|policy| StandingLog::prefetched(reader.root(), policy).map(drop));
        assert!(StandingLog::closed(&refused, GATE_CLOSED) && StandingLog::closed(&[counted], GATE_CLOSED));
    }
}
