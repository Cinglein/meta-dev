use std::time::Duration;

use super::Poll;
use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::watcher::gh::replies::Replies;
use crate::request::ci::watcher::gh::{Gh, Observation};
use crate::request::ci::watcher::note::Note;
use crate::request::output::Listed;

const NO_RUN: &str = "sha=45ae6b129220283516130564830df7c0e6c67fb1";
const FAILED: &str = "sha=e6554a9caf9e41b1c37dc15dd1e6896c276cb81d";
const PASSED: &str = "sha=0517805f95e8ed06ed80c08dd2624fcc86e8402a";
const OVERDUE: Option<()> = Some(());
const SUITES: [&str; 3] = [
    r#"{"checkSuites":{"nodes":[{"status":"COMPLETED","conclusion":"FAILURE","workflowRun":{"databaseId":1},"checkRuns":{"nodes":[{"name":"ci","conclusion":"FAILURE","steps":{"nodes":[]}}]},"otherJobs":{"nodes":[]}}]}}"#,
    r#"{"checkSuites":{"nodes":[{"status":"COMPLETED","conclusion":"FAILURE","workflowRun":{"databaseId":1},"checkRuns":{"nodes":[{"name":"ci","conclusion":"FAILURE","steps":{"nodes":[]}}]},"otherJobs":{"nodes":[{"name":"lint"}]}}]}}"#,
    r#"{"checkSuites":{"nodes":[{"status":"COMPLETED","conclusion":"FAILURE","workflowRun":{"databaseId":1},"checkRuns":{"nodes":[{"name":"ci","conclusion":"FAILURE","steps":{"nodes":[{"name":"t","conclusion":"FAILURE"}]}}]},"otherJobs":{"nodes":[]}}]}}"#,
];
const AWAITED: [&str; 2] = ["no run yet; Stop stays blocked until one appears or the wait is recorded", "run appeared"];
const ERRORS: [&str; 2] = ["gh failed: HTTP 502: Bad Gateway", "gh failed: HTTP 401: Bad credentials"];
const JOB_FAILED: &str = "ci failed; run `meta ci-log`";
const LONG: Duration = Duration::from_secs(510);
const HELD: &str = "CI 0517805f95e8ed06ed80c08dd2624fcc86e8402a: failed after 510s";
const CONCLUDED: [&str; 7] = [
    "failed after 510s: ci / Run cargo xtask ci , ci-gate / ",
    "passed after 510s",
    "absent after 510s: no run for this commit; no CI for this push",
    "timed-out after 510s",
    "unavailable after 510s: jobs never started; push a new commit to retry, then run `meta ci-wait`",
    "failed after 510s",
    "failed after 510s: ci / t; run `meta ci-log`",
];

#[test]
fn a_run_that_never_appears_is_noted_once_as_absent_and_one_appearing_later_is_noted_as_appeared() {
    let running = Observation::new(Vec::new(), None, Some(CiStatus::Running), None);
    let [none, running] = [Replies::observed(NO_RUN), running].map(Ok::<_, Listed>);
    let early = Poll::default().step(&none, None, None);
    let late = early.clone().step(&none, None, OVERDUE);
    let again = late.clone().step(&none, Some(CiStatus::Absent), OVERDUE);
    let seen = again.clone().step(&running, Some(CiStatus::Absent), OVERDUE);
    let still = seen.clone().step(&none, Some(CiStatus::Running), OVERDUE);
    let noted =
        [&early, &late, &again, &seen, &still].map(|poll| poll.fresh().map(|note| (note.line(), note.status())));
    let [absent, appeared] = AWAITED;
    assert_eq!(noted, [None, Some((absent, CiStatus::Absent)), None, Some((appeared, CiStatus::Running)), None]);
}

#[test]
fn gh_errors_are_noted_once_each_and_keep_a_failed_status_and_a_failed_job_says_run_ci_log() {
    let [down, denied] = ERRORS.map(|error| Err(Listed::new(error)));
    let once = Poll::default().step(&down, Some(CiStatus::Running), None);
    let twice = once.clone().step(&down, Some(CiStatus::Running), None);
    let held = twice.clone().step(&denied, Some(CiStatus::Failed), None);
    let failed = held.clone().step(&Ok(Replies::observed(FAILED)), Some(CiStatus::Failed), None);
    let noted = [&once, &twice, &held, &failed].map(|poll| poll.fresh().map(|note| (note.line(), note.status())));
    let [first, second] = ERRORS;
    let failed = [second, JOB_FAILED].map(|line| Some((line, CiStatus::Failed)));
    assert_eq!(noted, [Some((first, CiStatus::Running)), None, failed[0], failed[1]]);
}

#[test]
fn a_watch_concludes_failed_with_its_steps_else_finished_else_absent_else_timed_out_and_an_unchecked_one_not_at_all() {
    let [failed, passed] = [FAILED, PASSED].map(Replies::observed);
    let [unstarted, beside, started] = SUITES.map(|suites| Observation::from(&Gh::checks_in(suites).unwrap()));
    let lasts = [Some(&failed), Some(&passed), None, None, Some(&unstarted), Some(&beside), Some(&started)];
    let currents = [None, Some(CiStatus::Running), Some(CiStatus::Absent), Some(CiStatus::Running), None, None, None];
    let concluded = lasts.iter().zip(currents).filter_map(|(last, current)| Note::concluded(current, *last, LONG));
    let shown = concluded.zip(CONCLUDED).filter(|(note, line)| note.line().starts_with(line)).count();
    let [held, unchecked] = [CiStatus::Failed, CiStatus::Unchecked].map(|current| {
        let concluded = Note::concluded(Some(current), Some(&passed), LONG);
        concluded.map(|note| (note.status(), note.entry(&PASSED[4..].parse().unwrap())))
    });
    assert_eq!((shown, held, unchecked), (CONCLUDED.len(), Some((CiStatus::Failed, HELD.to_owned())), None));
}
