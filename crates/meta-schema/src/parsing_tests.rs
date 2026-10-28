use std::num::NonZeroU64;

use chrono::DateTime;
use meta_laws::{Laws, laws};
use proptest::prelude::{Strategy, any};
use proptest::sample::select;
use proptest::strategy::LazyJust;
use strum::IntoEnumIterator;

use crate::entity::{Concern, Task};
use crate::{Actor, Blocker, Division, DueTime, ExternalKind, Id, LevelName, PullRequest, Text, Tokens, TopDivision};

const READ: &str = "read by its parser";
const REFUSED: [&str; 3] = ["Major", "task-123", "performance-and-cost/ci_tests"];
const PULL_REQUEST: &str = "#12";
const BARE_PULL_REQUEST: &str = "12";
const DUE: &str = "2026-11-01T02:00:00+02:00";
const DUE_SHOWN: &str = "2026-11-01 00:00Z";
const TOP: &str = "performance-and-cost";
const NESTED: &str = "performance-and-cost/ci/tests";
const REVIEW: &str = "review";
const NOT_A_BLOCKER: &str = "nonsense";

laws! {
    texts_settle: Laws::settled::<Text> => any::<String>();
    texts_are_recorded: Laws::recorded::<Text> => any::<String>().prop_filter_map(READ, |text| text.parse().ok());
    actors_settle: Laws::settled::<Actor> => any::<String>();
    level_names_settle: Laws::settled::<LevelName> => any::<String>();
    divisions_settle: Laws::settled::<Division> => any::<String>();
    ids_name_actors: Laws::stored::<Actor> => LazyJust::new(|| Id::<Task>::generate().to_string());
    ids_name_levels: Laws::stored::<LevelName> => LazyJust::new(|| Id::<Task>::generate().to_string());
    ids_name_divisions: Laws::stored::<Division> => select(TopDivision::iter().collect::<Vec<_>>()).prop_map(|top| format!("{top}/{}", Id::<Task>::generate()));
    pull_requests_settle: Laws::settled::<PullRequest> => any::<NonZeroU64>().prop_map(|number| number.to_string());
    issues_block: Laws::shown::<Blocker> => LazyJust::new(|| Blocker::Discussion(Id::generate()));
    tasks_block: Laws::shown::<Blocker> => LazyJust::new(|| Blocker::Task(Id::generate()));
    tokens_are_stored: Laws::stored::<Tokens> => any::<NonZeroU64>().prop_map(|tokens| tokens.to_string());
    due_times_are_stored: Laws::stored::<DueTime> => any::<u32>().prop_map(|at| DateTime::from_timestamp(at.into(), 0).unwrap_or_default().to_rfc3339());
}

#[test]
fn a_blocker_reads_as_an_issue_a_task_or_an_external_kind_and_nothing_else() {
    let task = Id::<Task>::generate().to_string();
    assert!(task.parse::<Id<Concern>>().is_err_and(|failure| failure.to_string().contains(&task)));
    assert_eq!(REVIEW.parse::<Blocker>().ok(), Some(Blocker::External(ExternalKind::Review)));
    assert!(NOT_A_BLOCKER.parse::<Blocker>().is_err());
}

#[test]
fn a_division_id_starts_at_a_top_division_and_nests_by_slashes() {
    let [top, nested] = [TOP, NESTED].map(|id| id.parse::<Division>().unwrap());
    assert!(top.strictly_contains(&nested) && !nested.strictly_contains(&top) && !top.strictly_contains(&top));
}

#[test]
fn values_keep_their_meaning_when_shown_and_misspellings_are_refused() {
    let pull_request: PullRequest = BARE_PULL_REQUEST.parse().unwrap();
    assert_eq!([pull_request, PULL_REQUEST.parse().unwrap()].map(|read| read.to_string()), [PULL_REQUEST; 2]);
    assert_eq!(DUE.parse::<DueTime>().unwrap().to_string(), DUE_SHOWN);
    let [level, id, division] = REFUSED;
    assert!(
        level.parse::<LevelName>().is_err() && id.parse::<Id<Task>>().is_err() && division.parse::<Division>().is_err()
    );
}
