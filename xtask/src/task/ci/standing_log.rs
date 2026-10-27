use std::env;
use std::path::Path;

use meta_log::{GitLog, LogFailure, LogStore, Snapshot};
use meta_state::State;
use task::{ChangeLink, Ledger, Policy, Standing};

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

    pub fn prefetched(root: &Path, policy: &Policy) -> Result<(), Failure> {
        (policy.budgets().standing() || Stranding::rescaling(root, policy)?.is_some())
            .then(|| Self::git(root).read().map_err(Failure::GateLog))
            .transpose()
            .map(drop)
    }

    pub fn state(snapshot: Result<Snapshot, LogFailure>) -> Result<State, Failure> {
        let snapshot = snapshot.map_err(Failure::StandingLog)?;
        let records = snapshot.complete().map_err(Failure::StandingLog)?;
        Ok(State::fold(Some(records).filter(|records| !records.is_empty()).ok_or(Failure::NoLog)?))
    }

    pub fn counted(
        read: impl FnOnce() -> Result<Snapshot, LogFailure>,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<(), Failure> {
        let standing = policy.budgets().standing().then(|| Self::standing(read())).transpose()?;
        ledger.count(standing);
        Ok(())
    }

    fn standing(snapshot: Result<Snapshot, LogFailure>) -> Result<Standing, Failure> {
        let state = Self::state(snapshot)?;
        let link = ChangeLink::new(
            env::var(Self::PULL_REQUEST).ok().and_then(|number| number.parse().ok()),
            env::var(Self::HEAD).ok().and_then(|branch| branch.parse().ok()),
        );
        Ok(Standing::read(&state, &link))
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, LogStore, Snapshot};
    use settings::EmbeddedDefaults;
    use task::{Ledger, Policy};

    use super::{Failure, StandingLog, Stranding};
    use crate::task::ci::Ci;
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
        let policy = StandingLog::policy(STANDING);
        let offline = || Err(LogFailure::Contended);
        assert!(StandingLog::counted(offline, &Policy::defaults(), &mut Ledger::default()).is_ok());
        let refused = [
            StandingLog::counted(offline, &policy, &mut Ledger::default()),
            StandingLog::counted(|| Ok(Snapshot::default()), &policy, &mut Ledger::default()),
        ];
        assert!(StandingLog::closed(&refused, CLOSED));
    }

    #[test]
    fn the_gate_fetches_a_needed_log_first_so_the_isolated_run_reads_it_without_reaching_a_private_origin() {
        let [writer, reader] = [Fixture::new(), Fixture::new()];
        writer.shared_origin(reader.write(UNREACHABLE, TOP));
        reader.branch();
        StandingLog::git(writer.root()).append::<LogFailure>(&mut |_| Ok(Stranding::rated(TOP, MINOR))).unwrap();
        let [standing, scaled] = [STANDING, SCALED].map(StandingLog::policy);
        StandingLog::prefetched(reader.root(), &standing).unwrap();
        let log = StandingLog::git(reader.pointed(&reader.path(UNREACHABLE)).root());
        let [gated, actions] = [Some(reader.path(UNREACHABLE)), None].map(|record| Ci::new(None, None, record));
        let isolated = StandingLog::counted(|| gated.logged(&log), &standing, &mut Ledger::default());
        assert!(isolated.is_ok() && actions.logged(&log).is_err());
        let refused = [&standing, &scaled].map(|policy| StandingLog::prefetched(reader.root(), policy));
        assert!(StandingLog::closed(&refused, GATE_CLOSED));
        assert!(StandingLog::prefetched(reader.root(), &Policy::defaults()).is_ok());
    }
}
