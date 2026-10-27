use std::path::Path;

use meta_log::{GitLog, LogFailure, LogStore, RemoteName, Snapshot};
use meta_severity::{Scales, Strands};
use meta_state::State;
use task::{Branch, Policy};

use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;

pub struct Stranding;

impl Stranding {
    const REMOTE: &str = "origin";
    const GIT_ENTRY: &str = ".git";

    pub fn refuse_stranded(root: &Path, policy: &Policy) -> Result<(), Failure> {
        let before = PriorPolicy::read_at(&Revision::merge_base(root, policy.branch(Branch::Base))?).scales();
        let remote: RemoteName = Self::REMOTE.parse().unwrap_or_else(|never| match never {});
        let repository =
            root.ancestors().find(|dir| dir.join(Self::GIT_ENTRY).exists()).ok_or(Failure::NoRepository)?;
        Self::judged(&before, policy.scales(), GitLog::new(repository.to_path_buf(), remote).read())
    }

    fn judged(before: &Scales, now: &Scales, read: Result<Snapshot, LogFailure>) -> Result<(), Failure> {
        let snapshot = read.map_err(Failure::LogUnread)?;
        let state = State::fold(snapshot.complete().map_err(Failure::LogUnread)?);
        let already = before.stranded(&state);
        let fresh: Strands =
            now.stranded(&state).items().iter().filter(|item| !already.items().contains(item)).cloned().collect();
        (!fresh.items().is_empty()).then_some(fresh).map_or(Ok(()), |fresh| Err(Failure::Stranded(fresh)))
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, LogStore, MemoryLog};
    use meta_schema::{Event, NewConcern, Record};
    use meta_severity::Scales;

    use super::Stranding;

    const TOP: &str = "features-and-maintenance";
    const MAJOR: &str = "major";
    const OVERRIDDEN: &str =
        r#"{"features-and-maintenance": [{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]}"#;

    #[test]
    fn a_policy_change_that_strands_a_rated_item_fails_and_an_unread_log_fails_closed() {
        let log = MemoryLog::default();
        let concern =
            NewConcern::new(TOP.parse().unwrap(), TOP.parse().unwrap(), None, vec![]).rated(MAJOR.parse().unwrap());
        let record = Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern));
        log.append::<LogFailure>(&mut |_| Ok(record.clone())).unwrap();
        let [defaults, overridden] = [Scales::default(), serde_json::from_str(OVERRIDDEN).unwrap()];
        assert!(Stranding::judged(&defaults, &overridden, log.read()).unwrap_err().to_string().contains(MAJOR));
        assert!(Stranding::judged(&overridden, &overridden, log.read()).is_ok());
        assert!(Stranding::judged(&defaults, &defaults, Err(LogFailure::from(std::io::Error::other(TOP)))).is_err());
    }
}
