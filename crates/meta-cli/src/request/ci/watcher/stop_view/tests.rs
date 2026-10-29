use std::time::{Duration, Instant};

use meta_log::RemoteHeads;

use super::{StopView, Watched};
use crate::request::checkout::Checkout;
use crate::request::ci::watcher::Watcher;
use crate::request::ci::watcher::session_commits::SessionCommits;
use crate::request::ci::{Commit, ci_status::CiStatus};
use crate::request::output::Listed;
use crate::request::remote_failure::RemoteFailure;

const HEAD: &str = "f2e83b47f04f1aadaaf72b9d15074e933a513dc6";
const OLDER: &str = "0535038a5657a4872055f5ca2aa4ae04f811e419";
const ON_FEATURE: &str = "HEAD -> refs/heads/feature";
const DENIED: &str = "permission denied";
const SHOWN: [&str; 3] = [
    "reflog is off",
    "HEAD f2e83b47f04f1aadaaf72b9d15074e933a513dc6 is not pushed: push it, then run `meta ci-wait`; or ask the owner",
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
fn stop_names_unknown_commits_before_an_unpushed_head_and_leaves_passed_or_unresolved_runs_to_the_log_unless_late() {
    let [all, head_unpushed] = [Seen::pushed(&[]), Seen::pushed(&[HEAD])].map(Ok);
    let unknown = Err(Listed::new(SHOWN[0]));
    let shown = [
        Seen::judged(&unknown, Some(DENIED), &[]),
        Seen::judged(&head_unpushed, None, &[None]),
        Seen::judged(&all, None, &[Some(CiStatus::Passed), Some(CiStatus::TimedOut)]),
    ];
    assert_eq!(shown, SHOWN);
    let now = Instant::now();
    let reason = Some(Listed::new(DENIED));
    let stopped = [now, now + Duration::from_secs(1)].map(|deadline| Watcher::stopped(now, deadline, reason.clone()));
    assert_eq!(stopped, [Some(Listed::new(Watcher::OUT_OF_TIME)), reason]);
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
