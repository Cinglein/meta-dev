use crate::{Blocker, DueTime, ExternalKind, Id, PullRequest, Tokens};

const REFUSED_TOKENS: [&str; 4] = ["0", "-1", "1.5", "lots"];
const REFUSED_PULL_REQUESTS: [&str; 4] = ["0", "#0", "pr", "-3"];
const REFUSED_DUE: [&str; 2] = ["tomorrow", "2026-11-01"];
const PULL_REQUEST: &str = "#12";
const BARE_PULL_REQUEST: &str = "12";
const DUE: &str = "2026-11-01T02:00:00+02:00";
const DUE_SHOWN: &str = "2026-11-01 00:00Z";
const REVIEW: &str = "review";
const NOT_A_BLOCKER: &str = "nonsense";

#[test]
fn tokens_pull_requests_and_due_times_refuse_what_is_not_theirs() {
    assert!(REFUSED_TOKENS.iter().all(|text| text.parse::<Tokens>().is_err()));
    assert!(REFUSED_PULL_REQUESTS.iter().all(|text| text.parse::<PullRequest>().is_err()));
    assert!(REFUSED_DUE.iter().all(|text| text.parse::<DueTime>().is_err()));
}

#[test]
fn accepted_values_keep_their_meaning_when_shown() {
    let pull_request: PullRequest = BARE_PULL_REQUEST.parse().unwrap();
    assert_eq!(pull_request, PULL_REQUEST.parse().unwrap());
    assert_eq!(pull_request.to_string(), PULL_REQUEST);
    assert_eq!(DUE.parse::<DueTime>().unwrap().to_string(), DUE_SHOWN);
    assert_eq!(BARE_PULL_REQUEST.parse::<Tokens>().unwrap().to_string(), "12 tokens");
}

#[test]
fn a_blocker_reads_as_an_issue_a_task_or_an_external_kind_and_nothing_else() {
    let issue = Id::generate();
    let task = Id::generate();
    let parsed = [issue.to_string(), task.to_string(), REVIEW.to_owned()].map(|text| text.parse::<Blocker>().unwrap());
    let expected = [Blocker::Discussion(issue), Blocker::Task(task), Blocker::External(ExternalKind::Review)];
    assert_eq!(parsed, expected);
    assert!(NOT_A_BLOCKER.parse::<Blocker>().is_err());
    assert_eq!(parsed.map(|blocker| blocker.to_string().parse::<Blocker>().unwrap()), expected);
}
