use std::path::Path;

use meta_log::LogStore;
use meta_severity::{Scales, Strands};
use task::{Branch, Policy};

use crate::task::ci::standing_log::StandingLog;
use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;

pub struct Stranding;

impl Stranding {
    pub fn refuse_stranded(root: &Path, policy: &Policy) -> Result<(), Failure> {
        let before = PriorPolicy::read_at(&Revision::merge_base(root, policy.branch(Branch::Base))?).scales();
        Self::judged(&before, policy.scales(), &StandingLog::git(root))
    }

    fn judged(before: &Scales, now: &Scales, log: &impl LogStore) -> Result<(), Failure> {
        let state = StandingLog::state(log)?;
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
    const OVERRIDDEN: &str =
        r#"{"features-and-maintenance": [{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]}"#;

    #[test]
    fn a_policy_change_that_strands_a_rated_item_fails_and_an_empty_log_fails_closed() {
        let log = MemoryLog::default();
        let [defaults, overridden] = [Scales::default(), serde_json::from_str(OVERRIDDEN).unwrap()];
        let empty = Stranding::judged(&defaults, &defaults, &log).unwrap_err().to_string();
        let concern =
            NewConcern::new(TOP.parse().unwrap(), TOP.parse().unwrap(), None, vec![]).rated(MAJOR.parse().unwrap());
        let record = Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern));
        log.append::<LogFailure>(&mut |_| Ok(record.clone())).unwrap();
        assert!(Stranding::judged(&defaults, &overridden, &log).unwrap_err().to_string().contains(MAJOR));
        assert!(Stranding::judged(&overridden, &overridden, &log).is_ok() && empty.contains(CLOSED));
    }
}
