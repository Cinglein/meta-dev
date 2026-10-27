use std::collections::BTreeMap;

use meta_schema::{Division, Event, NewConcern, Record, Target};
use meta_state::State;

use crate::{Scale, Scales, SeverityFilter};

const FINER: &str = r#"{"features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "slow", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
]}"#;
const TOP: &str = "features-and-maintenance";
const NAV: &str = "features-and-maintenance/nav";
const LEVELS: [&str; 2] = ["critical", "retired-level"];
const STRANDED: &str = "stranded:";
const NOT_ON: &str = ", not a level of features-and-maintenance/nav's scale";
const COUNTED: [(&str, &str); 4] = [(TOP, "critical"), (NAV, "lost"), (TOP, "major"), (NAV, "slow")];

struct Repro {
    state: State,
    items: Vec<Target>,
}

impl Repro {
    fn rated_before_the_finer_scale() -> Repro {
        let concerns = LEVELS.map(|level| {
            NewConcern::new(NAV.parse().unwrap(), TOP.parse().unwrap(), None, vec![]).rated(level.parse().unwrap())
        });
        let items = concerns.iter().map(|concern| Target::Concern(concern.concern())).collect();
        let records: Vec<Record> = concerns
            .into_iter()
            .map(|concern| Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern)))
            .collect();
        Repro { state: State::fold(&records), items }
    }
}

#[test]
fn an_item_rated_before_a_finer_child_scale_still_counts_and_is_named() {
    let repro = Repro::rated_before_the_finer_scale();
    let finer = Scales::try_from(serde_json::from_str::<BTreeMap<Division, Scale>>(FINER).unwrap()).unwrap();
    let counted = COUNTED.map(|(division, level)| {
        SeverityFilter::AtLevel(level.parse().unwrap()).open(&finer, &division.parse().unwrap(), &repro.state)
    });
    assert_eq!(counted, [repro.items.clone(), repro.items.clone(), vec![], vec![]]);
    let named: Vec<String> = finer.stranded(&repro.state).iter().map(ToString::to_string).collect();
    let expected = repro.items.iter().zip(LEVELS).map(|(item, level)| format!("{STRANDED} {item} is {level}{NOT_ON}"));
    assert_eq!(named, expected.collect::<Vec<String>>());
    assert_eq!(Scales::default().stranded(&repro.state).len(), 1);
}
