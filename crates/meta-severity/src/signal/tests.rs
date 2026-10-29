use std::collections::BTreeMap;

use meta_schema::{Division, ItemKind, LevelName, Placement, TopDivision};

use crate::{ActionLevels, Evidence, Level, PromoteAfter, Scale, Scales, Signal, Subject, Thresholds};

type Observed = (Option<&'static str>, Evidence);

const LINES: [(usize, &str); 3] = [(2, "minor"), (10, "critical"), (5, "major")];
const AT_LINES: [Option<&str>; 3] = [None, Some("major"), Some("minor")];
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

impl Subject for (Placement, ItemKind) {
    fn placement(&self) -> Placement {
        self.0.clone()
    }

    fn kind(&self) -> ItemKind {
        self.1
    }
}

fn named(name: &str) -> LevelName {
    name.parse().unwrap()
}

fn observed(division: &Division, (level, evidence): &Observed) -> Signal<(Placement, ItemKind), ()> {
    Signal::new((Placement::new(division.clone()), ItemKind::Issue), level.map(named), evidence.clone(), ())
}

fn top(scales: &Scales) -> (Division, &Scale) {
    let division = Division::from(TopDivision::PerformanceAndCost);
    let scale = scales.scale(&division).unwrap();
    (division, scale)
}

#[test]
fn a_rating_never_falls_as_the_measure_grows_and_a_measure_at_a_line_stays_below_it() {
    let scales = Scales::default();
    let (_, scale) = top(&scales);
    let thresholds = Thresholds::on(scale, LINES.map(|(line, name)| (line, named(name))).to_vec()).unwrap();
    let rank = |measure: usize| thresholds.rate(&measure).and_then(|level| scale.index_of(level));
    let ranks: Vec<usize> = (0..=LINES[1].0 + 1).map(|measure| rank(measure).unwrap_or(scale.levels().len())).collect();
    assert!(ranks.is_sorted_by(|before, after| after <= before));
    assert_eq!(
        LINES.map(|(line, _)| thresholds.rate(&line).map(ToString::to_string)),
        AT_LINES.map(|at| at.map(String::from))
    );
}

#[test]
fn a_raise_needs_evidence_a_lowering_is_free_and_a_run_or_a_failed_remedy_promotes() {
    let scales = Scales::default();
    let (division, scale) = top(&scales);
    let after: PromoteAfter = serde_json::from_str("3").unwrap();
    let promoted = HISTORIES.map(|(history, _)| {
        let signals: Vec<_> = history.iter().map(|seen| observed(&division, seen)).collect();
        let level = after.promote(scale, &signals).map(|level| level.map(String::from).unwrap_or_default());
        level.unwrap_or_else(|failure| failure.to_string())
    });
    assert_eq!(promoted, HISTORIES.map(|(_, expected)| expected.to_owned()));
}

#[test]
fn missing_data_reaches_every_action_level_and_a_measured_level_only_its_own_and_above() {
    let scales = Scales::default();
    let (division, scale) = top(&scales);
    let acting = |line: &Level| -> ActionLevels {
        serde_json::from_value(serde_json::to_value(BTreeMap::from([(division.clone(), line.name())])).unwrap())
            .unwrap()
    };
    let minor = observed(&division, &(Some("minor"), Evidence::Measured));
    let reached = |level: Option<&LevelName>| {
        scale.levels().iter().map(|line| acting(line).acts(&scales, &division, level)).collect::<Vec<bool>>()
    };
    assert!(reached(None).iter().all(|acts| *acts));
    assert_eq!(reached(minor.level()), REACHED);
    assert_eq!(minor.triage(), (Placement::new(division.clone()), ItemKind::Issue, Some(&named("minor"))));
}
