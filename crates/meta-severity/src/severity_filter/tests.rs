use meta_schema::{EntryNote, Event, LevelName, NewConcern, NewIssue, Record, Target};
use meta_state::State;

use crate::declared::NAV_SCALE;
use crate::{Scales, SeverityFilter};

const TOP: &str = "features-and-maintenance";
const NAV: &str = "features-and-maintenance/nav";
const OFFLINE: &str = "features-and-maintenance/nav/offline";
const OTHER: &str = "performance-and-cost";
const ROOT_RATED: &str =
    r#"{"kind":"ConcernIdentified","concern":"concern-0000000000000009","text":"x","severity":"major"}"#;
const FILTERS: [(&str, Option<&str>, [bool; 5]); 6] = [
    (TOP, Some("critical"), [true, false, false, false, false]),
    (TOP, Some("major"), [false, false, true, false, false]),
    (TOP, None, [false, true, false, false, false]),
    (NAV, Some("lost"), [true, false, false, false, false]),
    (NAV, Some("critical"), [false; 5]),
    (OTHER, Some("critical"), [false, false, false, true, false]),
];
const UNKNOWN_LEVEL: &str = "lost is not a level of the division's scale";
const UNSCALED: &str = "an item at the root has no scale; place it in a division first";

struct Fixture {
    state: State,
    items: Vec<Target>,
}

impl Fixture {
    fn level(name: &str) -> LevelName {
        name.parse().unwrap()
    }

    fn concern(division: &str, level: Option<&str>) -> NewConcern {
        let concern = NewConcern::new(division.parse().unwrap(), TOP.parse().unwrap(), None, vec![]);
        level.map(Self::level).into_iter().fold(concern, NewConcern::rated)
    }

    fn built() -> Fixture {
        let [lost, unrated, retired] = [(NAV, Some("lost")), (OFFLINE, None), (NAV, Some("slow"))]
            .map(|(division, level)| Self::concern(division, level));
        let [major, other] = [(TOP, "major"), (OTHER, "critical")].map(|(division, level)| {
            NewIssue::new(division.parse().unwrap(), TOP.parse().unwrap()).rated(Self::level(level))
        });
        let root: Event = serde_json::from_str(ROOT_RATED).unwrap();
        let items = vec![
            Target::Concern(lost.concern()),
            Target::Concern(unrated.concern()),
            Target::Issue(major.issue()),
            Target::Issue(other.issue()),
            Target::Concern(retired.concern()),
        ];
        let retiring =
            Event::EntryRetired(EntryNote::new(Target::Concern(retired.concern()).entry(), TOP.parse().unwrap()));
        let events = [
            Event::ConcernIdentified(lost),
            Event::ConcernIdentified(unrated),
            Event::IssueReported(major),
            Event::IssueReported(other),
            Event::ConcernIdentified(retired),
            root,
            retiring,
        ];
        let records: Vec<Record> = events.into_iter().map(|event| Record::new(TOP.parse().unwrap(), event)).collect();
        Fixture { state: State::fold(&records), items }
    }
}

#[test]
fn a_filter_counts_open_items_in_its_division_at_a_rolled_up_level_and_new_items_take_scale_levels() {
    let fixture = Fixture::built();
    let scales = Scales::declared(NAV_SCALE).unwrap();
    let found = FILTERS.map(|(division, level, _)| {
        let filter = level.map_or(SeverityFilter::Unrated, |level| SeverityFilter::AtLevel(Fixture::level(level)));
        let open = filter.open(&scales, &division.parse().unwrap(), &fixture.state);
        fixture.items.iter().map(|item| open.contains(item)).collect::<Vec<bool>>()
    });
    assert_eq!(found, FILTERS.map(|(.., admitted)| admitted.to_vec()));
    let events = [
        Event::ConcernIdentified(Fixture::concern(NAV, Some("lost"))),
        Event::ConcernIdentified(Fixture::concern(TOP, None)),
        Event::IssueReported(NewIssue::new(TOP.parse().unwrap(), TOP.parse().unwrap()).rated(Fixture::level("lost"))),
        serde_json::from_str(ROOT_RATED).unwrap(),
    ];
    let admitted = events.map(|event| scales.admits(&event).map_err(|failure| failure.to_string()));
    assert_eq!(admitted, [Ok(()), Ok(()), Err(UNKNOWN_LEVEL.to_owned()), Err(UNSCALED.to_owned())]);
}
