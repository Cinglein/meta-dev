use std::env;
use std::path::Path;

use meta_log::{GitLog, LogStore};
use meta_state::State;
use task::{ChangeLink, Ledger, Policy, Standing};

use crate::task::failure::Failure;

pub struct StandingLog;

impl StandingLog {
    const REMOTE: &str = "origin";
    const PULL_REQUEST: &str = "PULL_REQUEST";
    const HEAD: &str = "GITHUB_HEAD_REF";

    pub fn git(root: &Path) -> GitLog {
        let Ok(remote) = Self::REMOTE.parse();
        GitLog::new(root.to_path_buf(), remote)
    }

    pub fn counted(log: &impl LogStore, policy: &Policy, ledger: &mut Ledger) -> Result<(), Failure> {
        let standing = policy.budgets().standing().then(|| Self::standing(log)).transpose()?;
        ledger.count(standing);
        Ok(())
    }

    fn standing(log: &impl LogStore) -> Result<Standing, Failure> {
        let snapshot = log.read().map_err(Failure::StandingLog)?;
        let records = snapshot.complete().map_err(Failure::StandingLog)?;
        let state = State::fold(Some(records).filter(|records| !records.is_empty()).ok_or(Failure::NoLog)?);
        let link = ChangeLink::new(
            env::var(Self::PULL_REQUEST).ok().and_then(|number| number.parse().ok()),
            env::var(Self::HEAD).ok().and_then(|branch| branch.parse().ok()),
        );
        Ok(Standing::read(&state, &link))
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, LogStore, MemoryLog, Snapshot};
    use meta_schema::Record;
    use settings::EmbeddedDefaults;
    use task::{Ledger, Policy};

    use super::StandingLog;

    const CLOSED: &str = "standing budgets fail closed: meta/log";

    struct Offline;

    impl LogStore for Offline {
        fn read(&self) -> Result<Snapshot, LogFailure> {
            Err(LogFailure::Contended)
        }

        fn append<Failure: From<LogFailure>>(
            &self,
            _: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
        ) -> Result<Record, Failure> {
            Err(LogFailure::Contended.into())
        }
    }

    #[test]
    fn an_unreadable_or_missing_log_fails_standing_budgets_closed() {
        let policy = Policy::defaults();
        let refused = [
            StandingLog::counted(&Offline, &policy, &mut Ledger::default()),
            StandingLog::counted(&MemoryLog::default(), &policy, &mut Ledger::default()),
        ];
        assert!(
            refused
                .iter()
                .all(|refused| refused.as_ref().err().is_some_and(|failure| failure.to_string().starts_with(CLOSED)))
        );
    }
}
