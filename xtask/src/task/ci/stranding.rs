use std::env;
use std::path::{Path, PathBuf};

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
        let databases = env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
            .unwrap_or_default()
            .join("git/db");
        Self::rescaling(root, policy, &databases)?
            .map_or(Ok(()), |before| Self::judged(&before, policy.scales(), read()))
    }

    pub fn rescaling(root: &Path, policy: &Policy, databases: &Path) -> Result<Option<Scales>, Failure> {
        let before = PriorPolicy::scales_at(&Revision::merge_base(root, policy.branch(Branch::Base))?, databases)?;
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
    use std::path::Path;

    use meta_log::Snapshot;
    use meta_schema::{Event, NewConcern, Record};
    use meta_severity::{Scale, Scales};
    use settings::EmbeddedDefaults;
    use task::Policy;

    use super::Stranding;
    use crate::task::fixture::Fixture;
    use crate::task::prior_policy::PriorPolicy;

    const PRIOR: &str = "[[scales.features-and-maintenance]]\nname = \"blocker\"\nmeaning = \"x\"\n\n[[scales.features-and-maintenance]]\nname = \"nuisance\"\nmeaning = \"x\"\n";
    const OLD_LEVELS: &str = r#"[{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]"#;
    const LOCK: &str = "Cargo.lock";
    const PIN: &str = "[[package]]\nname = \"meta-xtask\"\nsource = \"git+https://example.invalid/meta-dev?tag=v0#";
    const END: &str = "\"\n";
    const TIP: &str = "HEAD";
    const EARLIER: &str = "HEAD~1";
    const README: &str = "README";
    const LOST: &str = "0123abc";
    const MOVED: &str = "4567def";
    const OTHER: &str = "[[package]]\nname = \"other\"\n";
    const DECIDED: [Option<bool>; 7] = [Some(true), Some(false), None, Some(true), Some(false), None, Some(false)];
    const UNSOURCED: &str = "[[package]]\nname = \"meta-xtask\"\n";
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
        let [fixture, relevelled] = std::array::from_fn(|_| Fixture::new());
        let current = serde_json::to_string(&Scale::top_default()).unwrap();
        fixture.write(PriorPolicy::PATH, PRIOR).write(PriorPolicy::DEFAULT_LEVELS, &current).branch();
        relevelled.write(PriorPolicy::PATH, PRIOR).write(PriorPolicy::DEFAULT_LEVELS, OLD_LEVELS).branch();
        let databases = fixture.root();
        let uncapped = Policy::DEFAULTS.split(CAPS).next().unwrap_or_default();
        let same: Policy = toml::from_str(&format!("{uncapped}\n{PRIOR}")).unwrap();
        let before: Scales = serde_json::from_str(OVERRIDDEN).unwrap();
        let read = |fixture: &Fixture, policy: &Policy| Stranding::rescaling(fixture.root(), policy, databases);
        assert_eq!(read(&fixture, &Policy::defaults()).unwrap(), Some(before));
        assert_eq!(read(&fixture, &same).unwrap(), None);
        assert!(read(&relevelled, &Policy::defaults()).unwrap().is_some());
    }

    impl Stranding {
        fn database() -> Fixture {
            let database = Fixture::new();
            database.write(README, "x").committed();
            database.write(PriorPolicy::DEFAULT_LEVELS, OLD_LEVELS).committed();
            database
        }

        fn pin(commit: &str) -> String {
            format!("{PIN}{commit}{END}")
        }

        fn decided(databases: &Path, [then, now]: [&str; 2], policy_file: &str) -> Option<bool> {
            let consumer = Fixture::new();
            consumer.write(LOCK, then).write(policy_file, PRIOR).branch();
            consumer.write(LOCK, now);
            Stranding::rescaling(consumer.root(), &Policy::defaults(), databases).map(|prior| prior.is_some()).ok()
        }
    }

    #[test]
    fn prior_levels_are_looked_up_only_when_the_pin_moved_and_an_undecidable_case_fails_closed() {
        let database = Stranding::database();
        let [early, late] = [EARLIER, TIP].map(|name| database.revision(name).trim().to_owned());
        let databases = database.root().parent().unwrap();
        let [found, predates, missing, unmoved] =
            [&late, &early, &LOST.to_owned(), &LOST.to_owned()].map(|commit| Stranding::pin(commit));
        let moved = Stranding::pin(MOVED);
        let decided = |then: &str, now: &str, file| Stranding::decided(databases, [then, now], file);
        let judged = [
            decided(&found, &moved, PriorPolicy::PATH),
            decided(&predates, &moved, PriorPolicy::PATH),
            decided(&missing, &moved, PriorPolicy::PATH),
            decided(&unmoved, &unmoved, PriorPolicy::PATH),
            decided(OTHER, &found, PriorPolicy::PATH),
            decided(UNSOURCED, &found, PriorPolicy::PATH),
            decided(&found, &moved, README),
        ];
        assert_eq!(judged, DECIDED);
    }
}
