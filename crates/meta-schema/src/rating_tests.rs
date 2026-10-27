use crate::{Change, Event, NewIssue, Rating, Rerating};

const DIVISION: &str = "features-and-maintenance/nav";
const MAJOR: &str = "major";
const MINOR: &str = "minor";
const ISSUE: &str = "issue-0000000000000001";
const RAISED: &str = r#"{"kind":"ItemRated","item":"issue-0000000000000001","change":{"step":"raised","from":"minor","level":"major","scenario":"the map is lost offline"}}"#;
const RAISED_SHOWN: &str = "ItemRated issue-0000000000000001 major (raised from minor: the map is lost offline)";
const LOWERED_SHOWN: &str = "issue-0000000000000001 minor (lowered from major)";
const MOVED: [&str; 2] = [
    r#"{"kind":"ItemMoved","item":"issue-0000000000000001","division":"performance-and-cost","carry":{"step":"unrated"}}"#,
    r#"{"kind":"ItemMoved","item":"issue-0000000000000001","from":"features-and-maintenance","division":"performance-and-cost","carry":{"step":"changed","change":{"step":"lowered","from":"major","level":"minor"}}}"#,
];
const MOVED_SHOWN: [&str; 2] = [
    "ItemMoved issue-0000000000000001 to @performance-and-cost",
    "ItemMoved issue-0000000000000001 @features-and-maintenance to @performance-and-cost !minor (lowered from major)",
];

#[test]
fn a_rated_item_keeps_its_level_when_written() {
    let rated = NewIssue::new(DIVISION.parse().unwrap(), MAJOR.parse().unwrap()).rated(MAJOR.parse().unwrap());
    let read: NewIssue = serde_json::from_str(&serde_json::to_string(&rated).unwrap()).unwrap();
    assert!(
        read == rated
            && read.rating() == Rating::Rated(MAJOR.parse().unwrap())
            && read.to_string().ends_with(" !major")
    );
}

#[test]
fn a_change_or_move_reads_back_shows_its_step_and_knows_the_rating_it_replaces() {
    let moved = MOVED.map(|line| serde_json::from_str::<Event>(line).unwrap());
    assert_eq!(moved.each_ref().map(ToString::to_string), MOVED_SHOWN);
    assert_eq!(moved.each_ref().map(|event| serde_json::to_string(event).unwrap()), MOVED);
    let raised: Event = serde_json::from_str(RAISED).unwrap();
    assert_eq!([raised.to_string(), serde_json::to_string(&raised).unwrap()], [RAISED_SHOWN, RAISED]);
    let [minor, major] = [MINOR, MAJOR].map(|level| level.parse().unwrap());
    let lowered =
        Rerating::new(ISSUE.parse().unwrap(), DIVISION.parse().unwrap(), Change::Lowered { from: major, level: minor });
    let first = Change::First { level: lowered.change().level().clone() };
    assert_eq!([first.before(), lowered.change().before()], [Rating::Unrated, Rating::Rated(MAJOR.parse().unwrap())]);
    assert_eq!((lowered.to_string(), first.to_string()), (LOWERED_SHOWN.to_owned(), MINOR.to_owned()));
}
