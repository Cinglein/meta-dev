use meta_run::GhArgv;

use super::RawLog;
use crate::request::ci::watcher::gh::replies::Replies;
use crate::request::project::{CiSettings, Config, LogWindow};

const BUDGET_JOB: &str = "122967870084";
const KEPT: [&str; 2] = ["budget over its blocking line:", "workspace: test-time"];

const CLIPPY: &str = "ci\tRun cargo xtask ci\t2026-10-25T06:30:01.1000000Z    Compiling meta-log v0.0.0
ci\tRun cargo xtask ci\t2026-10-25T06:30:02.1000000Z     Checking meta-cli v0.0.0
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.1000000Z \u{1b}[1m\u{1b}[31merror\u{1b}[0m: this method chain can be written more clearly with `if .. else ..`
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.1000000Z   --> crates/meta-log/src/git_log/remote.rs:43:9
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.2000000Z error: could not compile `meta-log` (lib) due to 1 previous error
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.3000000Z ci: clippy 2.3s
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.4000000Z ci: clippy failed
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.5000000Z ##[error]Process completed with exit code 1.";
const WIDE: &str = "log_lines = 50\nlog_context = 1\n";
const NARROW: &str = "log_lines = 3\nlog_context = 0\n";
const HEADED_ERROR: &str = "== ci / Run cargo xtask ci ==\n    Checking meta-cli v0.0.0\nerror: this method chain";
const LOCATION: &str = "remote.rs:43:9";
const NOISE: &[&str] = &["Compiling", "2026-10-25T", "\u{1b}"];
const CAPPED: &str = "[kept 3 of 8 log lines; cut setup, passing output and anything beyond 3 lines]\n";

struct Limits;

impl Limits {
    fn excerpt(text: &str) -> String {
        let window: LogWindow = toml::from_str(text).unwrap();
        CLIPPY.parse::<RawLog>().unwrap().excerpt(&window).to_string()
    }
}

#[test]
fn keeps_error_blocks_without_setup_noise_timestamps_or_colour() {
    let excerpt = Limits::excerpt(WIDE);
    assert!(excerpt.contains(HEADED_ERROR) && excerpt.contains(LOCATION));
    assert!(NOISE.iter().all(|noise| !excerpt.contains(noise)));
}

#[test]
fn a_recorded_failed_log_keeps_the_lines_above_its_final_error_marker_with_the_default_window() {
    let logs = Replies::recorded(GhArgv::FailedLog);
    let log = logs.iter().find(|log| log.argv().to_string().contains(BUDGET_JOB)).unwrap();
    let settings = CiSettings::try_from(&toml::from_str::<Config>("").unwrap()).unwrap();
    let excerpt = log.out().parse::<RawLog>().unwrap().excerpt(settings.log_window()).to_string();
    assert!(KEPT.iter().all(|line| excerpt.contains(line)), "{excerpt}");
}

#[test]
fn caps_the_excerpt_and_says_how_much_was_kept() {
    assert!(Limits::excerpt(NARROW).ends_with(CAPPED));
}
