use meta_log::RemoteHeads;

use super::{StopView, Watched};
use crate::request::checkout::Checkout;
use crate::request::ci::watcher::session_commits::SessionCommits;
use crate::request::ci::{Commit, ci_status::CiStatus};
use crate::request::output::Listed;
use crate::request::remote_failure::RemoteFailure;

const HEAD: &str = "f2e83b47f04f1aadaaf72b9d15074e933a513dc6";
const OLDER: &str = "0535038a5657a4872055f5ca2aa4ae04f811e419";
const ON_FEATURE: &str = "HEAD -> refs/heads/feature";
const DENIED: &str = "permission denied";
const SHOWN: [&str; 6] = [
    "reflog is off",
    "CI state unwritable: permission denied; fix it or ask the owner",
    "HEAD f2e83b47f04f1aadaaf72b9d15074e933a513dc6 is not pushed: push it, then run `meta ci-wait`; or ask the owner",
    "CI not watched for 0535038a5657a4872055f5ca2aa4ae04f811e419: push again to watch it",
    "CI failed for your last push: fix it, push, then run `meta ci-wait`; or ask the owner",
    "",
];

struct Seen;

impl Seen {
    fn pushed(unpushed: &[&str]) -> SessionCommits {
        let [head, older] = [HEAD, OLDER].map(|sha| sha.parse::<Commit>().unwrap());
        let unpushed = unpushed.iter().map(|sha| sha.parse().unwrap()).collect();
        let checkout = Checkout::decorated(&head, ON_FEATURE).unwrap();
        SessionCommits::new(&[head, older], Some(unpushed), checkout, RemoteHeads::default())
    }

    fn judged(seen: &Result<SessionCommits, Listed>, unwritable: Option<&str>, at: &[Option<CiStatus>]) -> String {
        let pushed = seen.as_ref().map(|seen| seen.pushed().iter().cloned().zip(at.iter().copied()));
        let at = pushed.map(|pushed| pushed.map(|(commit, status)| Watched::new(commit, status)).collect());
        let view = StopView::new(seen, unwritable.map(str::to_owned), at.unwrap_or_default());
        view.reason().break_value().map(|reason| reason.to_string()).unwrap_or_default()
    }
}

#[test]
fn stop_names_the_first_of_unknown_commits_unwritable_state_unpushed_head_unwatched_and_known_ci() {
    let [all, head_unpushed] = [Seen::pushed(&[]), Seen::pushed(&[HEAD])].map(Ok);
    let unknown = Err(Listed::new(SHOWN[0]));
    let watched = [Some(CiStatus::Passed), Some(CiStatus::Failed)];
    let shown = [
        Seen::judged(&unknown, Some(DENIED), &[]),
        Seen::judged(&all, Some(DENIED), &[None]),
        Seen::judged(&head_unpushed, None, &[None]),
        Seen::judged(&all, None, &[Some(CiStatus::Passed), None]),
        Seen::judged(&all, None, &watched),
        Seen::judged(&all, None, &[Some(CiStatus::Passed), Some(CiStatus::TimedOut)]),
    ];
    assert_eq!(shown, SHOWN);
}

#[test]
fn stop_restarts_the_watcher_of_every_unfinished_commit_and_a_failed_remote_url_read_is_retried() {
    let older: Commit = OLDER.parse().unwrap();
    let statuses = [Some(CiStatus::Running), None, Some(CiStatus::Passed), Some(CiStatus::TimedOut)];
    let restarted = statuses.map(|status| Watched::new(older.clone(), status).restarted().cloned());
    assert_eq!(restarted, [Some(older.clone()), Some(older), None, None]);
    let [flaked, nameless] = [DENIED, ON_FEATURE].map(Listed::new);
    let retried = [RemoteFailure::GitFailed(flaked.clone()), RemoteFailure::NotGithub(nameless.clone())];
    assert_eq!(retried.map(RemoteFailure::retry), [Ok(flaked), Err(nameless)]);
}
