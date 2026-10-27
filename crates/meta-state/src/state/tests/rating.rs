use meta_schema::{Change, Event, LevelName, Placement, Rating, Rerating, Target};

use super::{Events, Items, OWNER, Script};
use crate::rated_item::RatedItem;
use crate::refusal::Refusal;
use crate::state::State;

const MAJOR: &str = "major";
const MINOR: &str = "minor";
const COST: &str = "performance-and-cost";
const ISSUE: &str = "issue-00000000000000bb";
const REPORTED: &str =
    r#"{"kind":"IssueReported","issue":"issue-00000000000000bb","text":"x","division":"features-and-maintenance"}"#;
const MOVED: &str = r#"{"kind":"ItemMoved","item":"issue-00000000000000bb","from":"features-and-maintenance","division":"performance-and-cost","carry":{"step":"unrated"}}"#;
const RATED: &str = r#"{"kind":"ItemRated","item":"issue-00000000000000bb","division":"features-and-maintenance","change":{"step":"first","level":"minor"}}"#;

struct Levels;

impl Levels {
    fn named(name: &str) -> LevelName {
        name.parse().unwrap()
    }

    fn rated(item: Target, change: Change) -> Event {
        Event::ItemRated(Rerating::new(item, Items::division(), change))
    }

    fn rating(script: &Script, item: Target) -> Option<Rating> {
        State::fold(&script.records).rated_item(item).map(|rated| rated.rating().clone())
    }
}

#[test]
fn a_change_applies_only_over_the_rating_it_replaces() {
    let concern = Items::concern();
    let issue = Items::issue().rated(Levels::named(MAJOR));
    let [item, reported] = [Target::Concern(concern.concern()), Target::Issue(issue.issue())];
    let first = Levels::rated(item, Change::First { level: Levels::named(MAJOR) });
    let opened =
        Script::default().then(OWNER, Event::ConcernIdentified(concern)).then(OWNER, Event::IssueReported(issue));
    let rated = opened.then(OWNER, first.clone());
    let lowered = Change::Lowered { from: Levels::named(MAJOR), level: Levels::named(MINOR) };
    let lower = rated.clone().then(OWNER, Levels::rated(reported, lowered.clone()));
    assert_eq!(
        [item, reported].map(|known| Levels::rating(&lower, known)),
        [MAJOR, MINOR].map(|level| Some(Rating::Rated(Levels::named(level))))
    );
    let stale = [rated.vet(OWNER, first), lower.vet(OWNER, Levels::rated(reported, lowered))];
    assert_eq!(stale, [MAJOR, MINOR].map(|level| Err(Refusal::RatedSince(Rating::Rated(Levels::named(level))))));
}

#[test]
fn a_move_and_a_rating_built_from_the_same_state_never_both_apply() {
    let event = |line: &str| serde_json::from_str::<Event>(line).unwrap();
    let reported = Script::default().then(OWNER, event(REPORTED));
    let [moved, rated] = [MOVED, RATED].map(|line| reported.clone().then(OWNER, event(line)));
    let minor = Rating::Rated(Levels::named(MINOR));
    let item = Target::Issue(ISSUE.parse().unwrap());
    let placed = RatedItem::new(item, Placement::new(COST.parse().unwrap()), Rating::Unrated);
    assert_eq!(State::fold(&moved.records).rated_item(item), Some(placed));
    let refusals = [moved.vet(OWNER, event(RATED)), rated.vet(OWNER, event(MOVED)), moved.vet(OWNER, event(MOVED))];
    assert_eq!(refusals, [Err(Refusal::MovedSince), Err(Refusal::RatedSince(minor)), Err(Refusal::MovedSince)]);
}

#[test]
fn retired_and_unknown_items_are_not_rated_and_only_open_items_are_listed() {
    let [kept, dropped] = [Items::concern(), Items::concern()];
    let issue = Items::issue().rated(Levels::named(MINOR));
    let [kept_item, dropped_item, issue_item] =
        [Target::Concern(kept.concern()), Target::Concern(dropped.concern()), Target::Issue(issue.issue())];
    let script = Script::default()
        .then(OWNER, Event::ConcernIdentified(kept))
        .then(OWNER, Event::ConcernIdentified(dropped))
        .then(OWNER, Event::IssueReported(issue))
        .then(OWNER, Events::retiring(dropped_item.entry()));
    let first = |item| Levels::rated(item, Change::First { level: Levels::named(MAJOR) });
    let ghost = Target::Issue(Items::issue().issue());
    let refusals = [script.vet(OWNER, first(dropped_item)), script.vet(OWNER, first(ghost))];
    assert_eq!(refusals, [Err(Refusal::Retired(dropped_item.entry())), Err(Refusal::UnknownTarget(ghost))]);
    let open: Vec<(Target, Rating)> =
        State::fold(&script.records).open_items().iter().map(|item| (item.target(), item.rating().clone())).collect();
    assert_eq!(open, [(kept_item, Rating::Unrated), (issue_item, Rating::Rated(Levels::named(MINOR)))]);
}
