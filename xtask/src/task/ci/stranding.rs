use std::path::Path;

use meta_log::{LogFailure, Snapshot};
use meta_severity::{Scales, Strands};
use task::{Branch, Policy};

use crate::task::ci::standing_log::StandingLog;
use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;

pub struct Stranding;

impl Stranding {
    pub fn refuse_stranded(
        root: &Path,
        policy: &Policy,
        read: impl FnOnce() -> Result<Snapshot, LogFailure>,
    ) -> Result<(), Failure> {
        Self::rescaling(root, policy)?.map_or(Ok(()), |before| Self::judged(&before, policy.scales(), read()))
    }

    pub fn rescaling(root: &Path, policy: &Policy) -> Result<Option<Scales>, Failure> {
        Ok(Some(PriorPolicy::read_at(&Revision::merge_base(root, policy.branch(Branch::Base))?)?.scales()))
    }

    fn judged(before: &Scales, now: &Scales, snapshot: Result<Snapshot, LogFailure>) -> Result<(), Failure> {
        let state = StandingLog::state(snapshot)?;
        let demoted = now.demotions_from(before, &state);
        (!demoted.items().is_empty()).then_some(demoted).map_or(Ok(()), |demoted| Err(Failure::Demoted(demoted)))?;
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
    const CLOSED: &str = "fail closed";
    const NAV: &str = "features-and-maintenance/nav";
    const CRITICAL: &str = "critical";
    const STRANDED: &str = "stranded:";
    const NOT_COMPARABLE: &str = "not comparable:";
    const FINER: &str = r#"{"features-and-maintenance/nav": [{"name": "lost", "meaning": "x", "parent": "critical"}, {"name": "late", "meaning": "x", "parent": "major"}, {"name": "typo", "meaning": "x", "parent": "minor"}]}"#;
    const OVERRIDDEN: &str =
        r#"{"features-and-maintenance": [{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]}"#;

    struct Logged;

    impl Logged {
        fn rated(division: &str, level: &str) -> MemoryLog {
            let log = MemoryLog::default();
            let concern = NewConcern::new(division.parse().unwrap(), TOP.parse().unwrap(), None, vec![])
                .rated(level.parse().unwrap());
            let record = Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern));
            log.append::<LogFailure>(&mut |_| Ok(record.clone())).unwrap();
            log
        }
    }

    #[test]
    fn a_policy_change_that_strands_lowers_or_reorders_fails_and_an_empty_log_fails_closed() {
        let [defaults, overridden, finer]: [Scales; 3] =
            [Scales::default(), serde_json::from_str(OVERRIDDEN).unwrap(), serde_json::from_str(FINER).unwrap()];
        let empty = Stranding::judged(&defaults, &defaults, MemoryLog::default().read()).unwrap_err().to_string();
        let [nested, top] = [Logged::rated(NAV, CRITICAL), Logged::rated(TOP, MAJOR)];
        let stranded = Stranding::judged(&defaults, &finer, nested.read()).unwrap_err().to_string();
        let reordered = Stranding::judged(&defaults, &overridden, top.read()).unwrap_err().to_string();
        assert!(stranded.contains(STRANDED) && reordered.contains(NOT_COMPARABLE) && empty.contains(CLOSED));
        assert!(Stranding::judged(&finer, &finer, nested.read()).is_ok());
    }
}
