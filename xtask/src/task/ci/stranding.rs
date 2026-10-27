use std::path::Path;

use meta_log::Snapshot;
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
        read: impl FnOnce() -> Result<Snapshot, Failure>,
    ) -> Result<(), Failure> {
        Self::rescaling(root, policy)?.map_or(Ok(()), |before| Self::judged(&before, policy.scales(), read()))
    }

    pub fn rescaling(root: &Path, policy: &Policy) -> Result<Option<Scales>, Failure> {
        let before = PriorPolicy::scales_at(&Revision::merge_base(root, policy.branch(Branch::Base))?)?;
        Ok(Some(before).filter(|before| before != policy.scales()))
    }

    fn judged(before: &Scales, now: &Scales, snapshot: Result<Snapshot, Failure>) -> Result<(), Failure> {
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
    use meta_log::Snapshot;
    use meta_schema::{Event, NewConcern, Record};
    use meta_severity::Scales;
    use settings::EmbeddedDefaults;
    use task::Policy;

    use super::Stranding;
    use crate::task::fixture::Fixture;
    use crate::task::prior_policy::PriorPolicy;

    const PRIOR: &str = "[[scales.features-and-maintenance]]\nname = \"blocker\"\nmeaning = \"x\"\n\n[[scales.features-and-maintenance]]\nname = \"nuisance\"\nmeaning = \"x\"\n";
    const OLD_LEVELS: &str = r#"[{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]"#;
    const TOP: &str = "features-and-maintenance";
    const MAJOR: &str = "major";
    const CAPS: &str = "\n[budgets.features-and-maintenance-critical]";
    const CLOSED: &str = "fail closed";
    const NAV: &str = "features-and-maintenance/nav";
    const CRITICAL: &str = "critical";
    const STRANDED: &str = "stranded:";
    const NOT_COMPARABLE: &str = "not comparable:";
    const FINER: &str = r#"{"features-and-maintenance/nav": [{"name": "lost", "meaning": "x", "parent": "critical"}, {"name": "late", "meaning": "x", "parent": "major"}, {"name": "typo", "meaning": "x", "parent": "minor"}]}"#;
    const OVERRIDDEN: &str =
        r#"{"features-and-maintenance": [{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]}"#;

    impl Stranding {
        pub fn rated(division: &str, level: &str) -> Record {
            let concern = NewConcern::new(division.parse().unwrap(), TOP.parse().unwrap(), None, vec![])
                .rated(level.parse().unwrap());
            Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern))
        }
    }

    #[test]
    fn a_policy_change_that_strands_lowers_or_reorders_fails_and_an_empty_log_fails_closed() {
        let [defaults, overridden, finer]: [Scales; 3] =
            [Scales::default(), serde_json::from_str(OVERRIDDEN).unwrap(), serde_json::from_str(FINER).unwrap()];
        let empty = Stranding::judged(&defaults, &defaults, Ok(Snapshot::default())).unwrap_err().to_string();
        let logged = |division, level| Ok(Snapshot::default().with_record(Stranding::rated(division, level)));
        let stranded = Stranding::judged(&defaults, &finer, logged(NAV, CRITICAL)).unwrap_err().to_string();
        let reordered = Stranding::judged(&defaults, &overridden, logged(TOP, MAJOR)).unwrap_err().to_string();
        assert!(stranded.contains(STRANDED) && reordered.contains(NOT_COMPARABLE) && empty.contains(CLOSED));
        assert!(Stranding::judged(&finer, &finer, logged(NAV, CRITICAL)).is_ok());
    }

    #[test]
    fn scales_changed_since_the_merge_base_policy_or_default_levels_are_judged_and_unchanged_ones_are_not() {
        let [fixture, relevelled] = [Fixture::new(), Fixture::new()];
        fixture.write(PriorPolicy::PATH, PRIOR).branch();
        relevelled.write(PriorPolicy::DEFAULT_LEVELS, OLD_LEVELS).branch();
        let uncapped = Policy::DEFAULTS.split(CAPS).next().unwrap_or_default();
        let same: Policy = toml::from_str(&format!("{uncapped}\n{PRIOR}")).unwrap();
        let before: Scales = serde_json::from_str(OVERRIDDEN).unwrap();
        assert_eq!(Stranding::rescaling(fixture.root(), &Policy::defaults()).unwrap(), Some(before));
        assert_eq!(Stranding::rescaling(fixture.root(), &same).unwrap(), None);
        assert!(Stranding::rescaling(relevelled.root(), &Policy::defaults()).unwrap().is_some());
    }
}
