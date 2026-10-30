use std::collections::BTreeMap;

use meta_schema::{Division, ItemKind, LevelName, Placement, Rating, Scenario, Text, TopDivision};

use crate::{ActionLevels, Evidence, PromoteAfter, Promoted, Reading, Scale, Scales, Signal, Subject, Thresholds};

type Observed = (&'static str, &'static str, &'static str);

const LINES: [(&str, usize); 3] = [("minor", 2), ("critical", 10), ("major", 5)];
const AT_LINES: [&str; 3] = ["clear", "major", "minor"];
const ODD: [&str; 3] =
    ["unrated", "bogus is not a level of the division's scale", "major needs a line above the next lower level's line"];
const RAISE: &str = "raising from minor needs evidence: a scenario, a measurement, or a failed remedy";
const HISTORIES: [(&[Observed], &str); 17] = [
    (&[("a", "clear", "measured"), ("a", "clear", "measured"), ("a", "clear", "measured")], "clear"),
    (&[("a", "clear", "measured"), ("a", "clear", "failed")], "clear"),
    (&[("a", "critical", "measured"), ("a", "minor", "failed")], "major"),
    (&[("a", "minor", "measured"), ("a", "critical", "absent")], RAISE),
    (&[("a", "minor", "measured"), ("a", "-", "absent"), ("a", "critical", "absent")], RAISE),
    (&[("a", "minor", "measured"), ("b", "critical", "measured"), ("a", "critical", "absent")], RAISE),
    (
        &[("a", "clear", "measured"), ("a", "minor", "absent")],
        "raising from clear needs evidence: a scenario, a measurement, or a failed remedy",
    ),
    (&[("a", "minor", "measured"), ("a", "critical", "measured")], "critical"),
    (&[("a", "minor", "measured"), ("a", "critical", "scenario")], "critical"),
    (&[("a", "critical", "measured"), ("a", "minor", "absent")], "minor"),
    (&[("a", "minor", "absent"), ("a", "-", "absent"), ("a", "minor", "absent"), ("a", "minor", "absent")], "major"),
    (&[("a", "minor", "absent"), ("b", "minor", "absent"), ("a", "minor", "absent")], "minor"),
    (&[("a", "critical", "measured"), ("a", "-", "absent")], "unrated"),
    (&[("a", "major", "measured"), ("a", "major", "failed")], "critical"),
    (&[("a", "minor", "measured"), ("a", "major", "failed")], "major"),
    (&[("a", "minor", "absent"), ("a", "minor", "absent"), ("a", "minor", "failed")], "major"),
    (&[("a", "critical", "absent"), ("a", "critical", "absent"), ("a", "critical", "absent")], "critical"),
];
const MOST: usize = 11;
const BOGUS: [(&str, usize); 1] = [("bogus", 1)];
const FLAT: [(&str, usize); 2] = [("major", 1), ("minor", 1)];
const READINGS: [&str; 3] = ["-", "clear", "minor"];
const REACHED: [[bool; 3]; 3] = [[true, true, true], [false, false, false], [false, false, true]];
const NESTED_DIVISION: &str = "performance-and-cost/x";
const NESTED: [bool; 3] = [true, false, true];
const OVERRIDES: [(&str, &str); 2] = [("performance-and-cost", "major"), (NESTED_DIVISION, "minor")];
const TRIAGED: Observed = ("a", "minor", "measured");

impl Subject for &str {
    fn placement(&self) -> Placement {
        Placement::new(Scales::costs())
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

impl Reading {
    fn named(name: &str) -> Reading {
        match name {
            "-" => Reading::from(Rating::Unrated),
            "clear" => Reading::Clear,
            level => Reading::from(Rating::Rated(level.parse().unwrap())),
        }
    }
}

impl Signal<&'static str, ()> {
    fn observed(subject: &'static str, reading: &str, evidence: &str) -> Signal<&'static str, ()> {
        let evidence = match evidence {
            "measured" => Evidence::Measured,
            "failed" => Evidence::FailedRemedy,
            "scenario" => Evidence::Scenario(Scenario::from(evidence.parse::<Text>().unwrap())),
            _ => Evidence::Absent,
        };
        Signal::new(subject, Reading::named(reading), evidence, ())
    }
}

#[test]
fn a_rating_never_falls_as_the_measure_grows_a_measure_at_a_line_stays_below_it_and_bad_lines_are_refused() {
    let scales = Scales::default();
    let scale = scales.top();
    let lined = |lines: &[(&str, usize)]| {
        Thresholds::from(
            lines.iter().map(|(name, line)| (name.parse::<LevelName>().unwrap(), *line)).collect::<BTreeMap<_, _>>(),
        )
    };
    let thresholds = lined(&LINES);
    let rated = |measure: usize| thresholds.rate(scale, Some(&measure)).unwrap();
    let ranks: Vec<_> = (0..=MOST).map(|measure| rated(measure).index_on(scale).unwrap()).collect();
    assert!(ranks.is_sorted_by(|before, after| after <= before));
    assert_eq!(LINES.map(|(_, line)| rated(line).to_string()), AT_LINES);
    let bad = [&LINES[..], &BOGUS, &FLAT].map(|lines| lined(lines).rate(scale, None));
    assert_eq!(bad.map(|rated| rated.map_or_else(|failure| failure.to_string(), |reading| reading.to_string())), ODD);
}

#[test]
fn a_raise_needs_evidence_over_gaps_and_other_subjects_and_a_run_or_a_failed_remedy_promotes_one_step() {
    let scales = Scales::default();
    let after: PromoteAfter = serde_json::from_str("3").unwrap();
    let promoted = HISTORIES.map(|(history, _)| {
        let signals: Vec<_> =
            history.iter().map(|(subject, reading, evidence)| Signal::observed(subject, reading, evidence)).collect();
        let (latest, earlier) = signals.split_last().unwrap();
        let triage = latest.triage(after, scales.top(), earlier);
        triage.map_or_else(|failure| failure.to_string(), |triage| triage.promoted().to_string())
    });
    assert_eq!(promoted, HISTORIES.map(|(_, expected)| expected.to_owned()));
    assert!(serde_json::from_str::<PromoteAfter>("0").is_err());
}

#[test]
fn missing_data_reaches_every_action_level_clear_reaches_none_and_a_nested_division_acts_at_its_top() {
    let scales = Scales::default();
    let readings = READINGS.map(|name| Promoted::new(Reading::named(name)));
    let acting = |lines: &[(&str, &str)]| -> ActionLevels {
        serde_json::from_value(serde_json::to_value(lines.iter().copied().collect::<BTreeMap<_, _>>()).unwrap())
            .unwrap()
    };
    let named: Vec<String> = scales.top().levels().iter().map(|line| line.name().to_string()).collect();
    let lines: Vec<ActionLevels> = named.iter().map(|line| acting(&[(OVERRIDES[0].0, line.as_str())])).collect();
    let reached = readings.iter().map(|reading| lines.iter().map(|at| at.acts(&scales, &Scales::costs(), reading)));
    assert_eq!(reached.map(Iterator::collect::<Vec<bool>>).collect::<Vec<_>>(), REACHED);
    let nested: Division = NESTED_DIVISION.parse().unwrap();
    let overridden = acting(&OVERRIDES);
    assert_eq!(readings.map(|reading| overridden.acts(&scales, &nested, &reading)), NESTED);
    let after: PromoteAfter = serde_json::from_str("3").unwrap();
    let triage = Signal::observed(TRIAGED.0, TRIAGED.1, TRIAGED.2).triage(after, scales.top(), &[]).unwrap();
    assert_eq!(
        (triage.placement().division(), triage.kind(), triage.promoted().reading()),
        (Some(&Scales::costs()), ItemKind::Issue, &Reading::named(TRIAGED.1))
    );
}
