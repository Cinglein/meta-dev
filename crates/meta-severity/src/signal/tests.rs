use std::collections::BTreeMap;

use meta_schema::{Division, ItemKind, LevelName, Placement, TopDivision};

use crate::{ActionLevels, Evidence, Level, PromoteAfter, Scale, Scales, Signal, Subject, Thresholds};

const LINES: [(&str, usize); 3] = [("minor", 2), ("critical", 10), ("major", 5)];
const AT_LINES: [Option<&str>; 3] = [None, Some("major"), Some("minor")];
type Observed = (Option<&'static str>, Evidence);

const HISTORIES: [(&[Observed], &str); 7] = [
    (
        &[(Some("minor"), Evidence::Measured), (Some("critical"), Evidence::Absent)],
        "raising from minor needs evidence: a measure, a failed remedy, or a scenario",
    ),
    (&[(Some("minor"), Evidence::Measured), (Some("critical"), Evidence::Measured)], "critical"),
    (&[(Some("critical"), Evidence::Measured), (Some("minor"), Evidence::Absent)], "minor"),
    (&[(Some("minor"), Evidence::Measured), (Some("minor"), Evidence::Measured)], "minor"),
    (
        &[(Some("minor"), Evidence::Absent), (Some("minor"), Evidence::Absent), (Some("minor"), Evidence::Absent)],
        "major",
    ),
    (&[(Some("major"), Evidence::Measured), (Some("major"), Evidence::FailedRemedy)], "critical"),
    (&[(Some("critical"), Evidence::Measured), (None, Evidence::Absent)], ""),
];
const REACHED: [bool; 3] = [false, false, true];

impl Subject for Placement {
    fn placement(&self) -> Placement {
        self.clone()
    }

    fn kind(&self) -> ItemKind {
        ItemKind::Issue
    }
}

impl Scales {
    fn costs() -> Division {
        Division::from(TopDivision::PerformanceAndCost)
    }

    fn top(&self) -> &Scale {
        self.scale(&Scales::costs()).unwrap()
    }
}

impl Signal<Placement, ()> {
    fn observed(level: Option<&str>, evidence: &Evidence) -> Signal<Placement, ()> {
        let level = level.map(|name| name.parse().unwrap());
        Signal::new(Placement::new(Scales::costs()), level, evidence.clone(), ())
    }
}

#[test]
fn a_rating_never_falls_as_the_measure_grows_and_a_measure_at_a_line_stays_below_it() {
    let scales = Scales::default();
    let scale = scales.top();
    let lines: BTreeMap<LevelName, usize> = LINES.map(|(name, line)| (name.parse().unwrap(), line)).into();
    let thresholds = Thresholds::from(lines);
    let rated = |measure: usize| thresholds.rate(scale, &measure).unwrap().map(String::from);
    let rank = |measure: usize| rated(measure).and_then(|level| scale.index_of(&level.parse().unwrap()));
    let ranks: Vec<usize> = (0..=LINES[1].1 + 1).map(|measure| rank(measure).unwrap_or(scale.levels().len())).collect();
    assert!(ranks.is_sorted_by(|before, after| after <= before));
    assert_eq!(LINES.map(|(_, line)| rated(line)), AT_LINES.map(|at| at.map(String::from)));
}

#[test]
fn a_raise_needs_evidence_a_lowering_is_free_and_a_run_or_a_failed_remedy_promotes() {
    let scales = Scales::default();
    let after: PromoteAfter = serde_json::from_str("3").unwrap();
    let promoted = HISTORIES.map(|(history, _)| {
        let signals: Vec<_> = history.iter().map(|(level, evidence)| Signal::observed(*level, evidence)).collect();
        let level = after.promote(scales.top(), &signals).map(|level| level.map(String::from).unwrap_or_default());
        level.unwrap_or_else(|failure| failure.to_string())
    });
    assert_eq!(promoted, HISTORIES.map(|(_, expected)| expected.to_owned()));
}

#[test]
fn missing_data_reaches_every_action_level_and_a_measured_level_only_its_own_and_above() {
    let scales = Scales::default();
    let acting = |line: &Level| -> ActionLevels {
        serde_json::from_value(serde_json::to_value(BTreeMap::from([(Scales::costs(), line.name())])).unwrap()).unwrap()
    };
    let minor = Signal::observed(Some("minor"), &Evidence::Measured);
    let reached = |level: Option<&LevelName>| {
        let lines = scales.top().levels().iter();
        lines.map(|line| acting(line).acts(&scales, &Scales::costs(), level)).collect::<Vec<bool>>()
    };
    assert!(reached(None).iter().all(|acts| *acts));
    assert_eq!(reached(minor.level()), REACHED);
    let triage = minor.triage();
    assert_eq!(
        (triage.placement().division(), triage.kind(), triage.level()),
        (Some(&Scales::costs()), ItemKind::Issue, minor.level())
    );
}
