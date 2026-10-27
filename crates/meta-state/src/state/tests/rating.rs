use meta_schema::{Change, Event, LevelName, Rating, Rerating, Target};

use super::{Events, Items, OWNER, Script};
use crate::refusal::Refusal;
use crate::state::State;

const MAJOR: &str = "major";
const MINOR: &str = "minor";

struct Levels;

impl Levels {
    fn named(name: &str) -> LevelName {
        name.parse().unwrap()
    }

    fn rated(item: Target, change: Change) -> Event {
        Event::ItemRated(Rerating::new(item, change))
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
