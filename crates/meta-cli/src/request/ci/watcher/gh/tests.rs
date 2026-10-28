use std::time::{Duration, Instant};

use meta_golden::GoldenFailure;
use meta_run::GhArgv;

use super::replies::Replies;
use super::{Gh, Observation};
use crate::request::project::{CiSettings, Config};

const FAILED: &str = "sha=e6554a9caf9e41b1c37dc15dd1e6896c276cb81d";
const DOWN: &str = "HTTP 502: Bad Gateway";
const SHOWN: [&str; 3] =
    ["== ci / Run cargo xtask ci  ==\n##[error]", "== ci-gate / Run echo", "\ngh failed: HTTP 502"];
const LIMITED: &str = "[ci]\ngh_millis = 400\n";
const WITHIN: Duration = Duration::from_secs(30);

#[test]
fn every_recorded_commit_checks_reply_golden_view_is_its_observation() -> Result<(), GoldenFailure> {
    let replies = Replies::recorded(GhArgv::CommitChecks);
    let observed = |reply: &meta_golden::Recording| Gh::checks_in(reply.out()).map(|checks| Observation::from(&checks));
    let viewed = replies.iter().map(|reply| reply.viewed(&observed(reply)));
    assert_ne!(viewed.collect::<Result<Vec<()>, _>>()?, []);
    Ok(())
}

#[test]
fn the_excerpt_shows_the_failed_steps_of_every_recorded_log_and_names_a_failed_download() {
    let checks = [Gh::checks_in(Replies::about(FAILED).out())];
    let logs = Replies::recorded(GhArgv::FailedLog).into_iter().map(|log| Ok(log.out().to_owned()));
    let logs: Vec<_> = logs.chain([Err(Gh::failed(DOWN))]).collect();
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let shown = Gh::excerpt(&checks, &logs, settings.log_window()).to_string();
    assert!(SHOWN.iter().all(|part| shown.contains(part)), "{shown}");
}

#[test]
fn a_log_download_may_outlast_the_call_limit_that_bounds_each_checks_query() {
    let settings = CiSettings::try_from(&toml::from_str::<Config>(LIMITED).unwrap()).unwrap();
    let now = Instant::now();
    let deadline = now + WITHIN;
    let gh = Gh::new(&settings, std::path::Path::new(""), deadline);
    let limited = now + settings.gh_limit();
    assert_eq!([GhArgv::FailedLog, GhArgv::CommitChecks].map(|call| gh.deadline_of(call, now)), [deadline, limited]);
}
