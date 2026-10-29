use std::fmt::Debug;
use std::num::NonZeroU64;

use chrono::DateTime;
use meta_laws::{Laws, Reading, laws};
use proptest::prelude::{Strategy, any};
use proptest::sample::select;
use proptest::string::{RegexGeneratorStrategy, string_regex};
use strum::IntoEnumIterator;

use crate::entity::{Concern, Entity, Issue, Task};
use crate::unrecognised::Unrecognised;
use crate::{
    Actor, Blocker, CommitId, Division, DueTime, ExternalKind, Id, LevelName, PullRequest, Text, Tokens, TopDivision,
};

const TEXTS: &str = r"\PC{0,8}[^\s\p{C}]\PC{0,8}";
const ACTORS: &str = "[a-z0-9][a-z0-9_.-]{0,12}";
const LEVEL_NAMES: &str = "[a-z0-9-]{0,6}[a-z][a-z0-9-]{0,6}";
const DIVISIONS: &str = "(features-and-maintenance|performance-and-cost|architecture-and-planning)(/[a-z0-9-]{0,3}[a-z0-9][a-z0-9-]{0,3}){0,3}";
const HEAD: [Reading; 1] = [Reading::verbatim("6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94")];
const COUNTED: [Reading; 1] = [Reading::new("12", "12 tokens")];
const NUMBERED: [Reading; 2] = [Reading::new("12", "#12"), Reading::verbatim("#12")];
const DUE: [Reading; 1] = [Reading::new("2026-11-01T02:00:00+02:00", "2026-11-01 00:00Z")];
const COMMITS: [&str; 4] = [
    "6c1ea3b",
    "6C1EA3B4D9F1F48D98517B891F4283D6F0E55B94",
    "6c1ea3b4d9f1f48d98517b891f4283d6f0e55b9g",
    "6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 ",
];
const TOKENS: [&str; 4] = ["0", "-1", "1.5", "lots"];
const PULL_REQUESTS: [&str; 4] = ["0", "#0", "pr", "-3"];
const DUE_TIMES: [&str; 2] = ["tomorrow", "2026-11-01"];
const EXPECTED: [&str; 4] = ["a full commit id", "a positive token count", "a pull request number", "an RFC 3339 time"];
const TOP: &str = "performance-and-cost";
const NESTED: &str = "performance-and-cost/ci/tests";
const REVIEW: &str = "review";
const NOT_A_BLOCKER: &str = "nonsense";

struct Drawn;

impl Drawn {
    fn spelled(grammar: &str) -> RegexGeneratorStrategy<String> {
        string_regex(grammar).unwrap()
    }

    fn numbered<Kind: Entity + Debug>() -> impl Strategy<Value = Id<Kind>> {
        any::<u64>().prop_map(|raw| format!("{}-{raw:016x}", Kind::PREFIX).parse().unwrap())
    }

    fn shown<Kind: Entity + Debug>() -> impl Strategy<Value = String> {
        Self::numbered::<Kind>().prop_map(|id| id.to_string())
    }
}

laws! {
    texts_settle: Laws::settled::<Text> => Drawn::spelled(TEXTS);
    texts_are_stored: Laws::stored::<Text> => Drawn::spelled(TEXTS);
    actors_settle: Laws::settled::<Actor> => Drawn::spelled(ACTORS);
    level_names_settle: Laws::settled::<LevelName> => Drawn::spelled(LEVEL_NAMES);
    divisions_settle: Laws::settled::<Division> => Drawn::spelled(DIVISIONS);
    ids_name_actors: Laws::stored::<Actor> => Drawn::shown::<Task>();
    ids_name_levels: Laws::stored::<LevelName> => Drawn::shown::<Task>();
    ids_name_divisions: Laws::stored::<Division> => (select(TopDivision::iter().collect::<Vec<_>>()), Drawn::shown::<Task>()).prop_map(|(top, id)| format!("{top}/{id}"));
    pull_requests_settle: Laws::settled::<PullRequest> => any::<NonZeroU64>().prop_map(|number| number.to_string());
    issues_block: Laws::shown::<Blocker> => Drawn::numbered::<Issue>().prop_map(Blocker::Discussion);
    tasks_block: Laws::shown::<Blocker> => Drawn::numbered::<Task>().prop_map(Blocker::Task);
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
fn commits_tokens_pull_requests_and_due_times_show_their_meaning_and_refuse_what_is_not_theirs() {
    let [commit, tokens, pull_request, due] =
        EXPECTED.map(|expected| move |text: &str| Unrecognised::new(expected, text));
    let misread = [
        Laws::misread::<CommitId>(&HEAD, &COMMITS, commit),
        Laws::misread::<Tokens>(&COUNTED, &TOKENS, tokens),
        Laws::misread::<PullRequest>(&NUMBERED, &PULL_REQUESTS, pull_request),
        Laws::misread::<DueTime>(&DUE, &DUE_TIMES, due),
    ];
    assert_eq!(misread.concat(), Laws::NONE_MISREAD);
}
