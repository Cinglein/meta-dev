use std::time::Duration;

use meta_golden::Golden;
use meta_schema::PullRequest;

use crate::live_pull::MergeState;
use crate::{GhRefusal, Patience, Stop};

const DENIED: &str = " HTTP 403: denied \n";
const LIMIT: Duration = Duration::from_secs(60);
const PAST: Duration = Duration::from_millis(1);

#[test]
fn a_stop_names_its_evidence_and_patience_lasts_through_its_limit() {
    let pull_request: PullRequest = "7".parse().unwrap();
    let refusal = GhRefusal::new(DENIED);
    let stops =
        [Stop::Unread { pull_request, refusal }, Stop::NotMergeable { pull_request, state: MergeState::Blocked }];
    Golden::for_test(module_path!(), "stops").debugged(&stops.map(|stop| stop.to_string())).unwrap();
    assert_eq!([LIMIT, LIMIT + PAST].map(|waited| Patience::new(waited, LIMIT).spent()), [None, Some(())]);
}
