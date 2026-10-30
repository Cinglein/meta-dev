use std::time::{Duration, Instant};

use meta_run::GhArgv;

use super::commit_checks::CommitChecks;
use super::replies::Replies;
use super::{Gh, Observation};
use crate::request::project::{CiSettings, Config};

const FAILED: &str = "sha=e6554a9caf9e41b1c37dc15dd1e6896c276cb81d";
const DOWN: &str = "HTTP 502: Bad Gateway";
const SHOWN: [&str; 4] = [
    "== ci / Run cargo xtask ci  ==\n",
    "budget over its blocking line:",
    "== ci-gate / Run echo",
    "\ngh failed: HTTP 502",
];
const LIMITED: &str = "[ci]\ngh_millis = 400\n";
const WITHIN: Duration = Duration::from_secs(30);
const FETCHED: usize = 1;
const LEARNED: [Option<u64>; 4] = [Some(432), Some(432), Some(422), None];

#[test]
fn recorded_checks_name_exactly_the_failed_jobs_whose_logs_were_fetched_and_expect_the_median_ancestor_run() {
    let replies = Replies::recorded(GhArgv::CommitChecks).into_iter().map(|reply| Gh::checks_in(reply.out()));
    let checks: Vec<CommitChecks> = replies.collect::<Result<_, _>>().unwrap();
    let expected: Vec<_> = checks.iter().map(|checks| Observation::from(checks).expected()).collect();
    let fetched: Vec<String> = Replies::recorded(GhArgv::FailedLog).iter().map(|log| log.argv().to_string()).collect();
    let named = checks[FETCHED].failed_job_ids();
    let quoted = named.iter().map(|job| format!("\"{}\"", job.to_string_lossy()));
    assert!(named.len() == fetched.len() && quoted.zip(&fetched).all(|(job, argv)| argv.contains(&job)), "{named:?}");
    assert_eq!(expected, LEARNED.map(|secs| secs.map(Duration::from_secs)));
}

#[test]
fn the_excerpt_shows_every_recorded_failed_log_and_a_failed_download_whose_call_may_outlast_the_checks_limit() {
    let checks =
        Replies::recorded(GhArgv::CommitChecks).into_iter().filter(|reply| reply.argv().to_string().contains(FAILED));
    let checks: Vec<_> = checks.map(|reply| Gh::checks_in(reply.out())).collect();
    let logs = Replies::recorded(GhArgv::FailedLog).into_iter().map(|log| Ok(log.out().to_owned()));
    let logs: Vec<_> = logs.chain([Err(Gh::failed(DOWN))]).collect();
    let settings = CiSettings::try_from(&toml::from_str::<Config>(LIMITED).unwrap()).unwrap();
    let shown = Gh::excerpt(&checks, &logs, settings.log_window()).to_string();
    let now = Instant::now();
    let gh = Gh::new(&settings, std::path::Path::new(""), now + WITHIN);
    let deadlines = [GhArgv::FailedLog, GhArgv::CommitChecks].map(|call| gh.deadline_of(call, now));
    assert!(SHOWN.iter().all(|part| shown.matches(part).count() == 1), "{shown}");
    assert_eq!(deadlines, [now + WITHIN, now + settings.gh_limit()]);
}
