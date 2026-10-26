use std::time::Duration;

use super::{Beat, SETTINGS};
use crate::request::project::TickLimit;

const OVERLONG_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 300\nstale_seconds = 900";
const SHORT_STALE: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 300";
const ZERO_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 0\nstale_seconds = 900";
const UNKNOWN_KEY: &str = "[heartbeat]\ninterval = 300";
const INTERVAL: &str = "300";
const TICK: Duration = Duration::from_mins(1);
const STALE: Duration = Duration::from_mins(15);
const WITHIN: Duration = Duration::from_mins(14);

#[test]
fn settings_read_the_configured_limits() {
    let valid = Beat::settings(SETTINGS).unwrap();
    assert_eq!(valid.interval().to_string(), INTERVAL);
    assert_eq!(valid.tick_limit(), TickLimit::from(TICK));
    assert_eq!(valid.stale_limit().passed_by(STALE), None);
    assert_eq!(valid.stale_limit().passed_by(STALE + WITHIN), Some(STALE));
}

#[test]
fn settings_need_a_tick_shorter_than_the_interval_which_is_shorter_than_the_stale_limit() {
    let refused = [OVERLONG_TICK, SHORT_STALE, ZERO_TICK, UNKNOWN_KEY].map(|text| Beat::settings(text).is_err());
    assert!(refused.iter().all(|refused| *refused));
}

#[test]
fn absent_settings_take_the_defaults() {
    let defaults = Beat::settings("").unwrap();
    assert_eq!(defaults.interval().to_string(), INTERVAL);
    assert_eq!(defaults.stale_limit().passed_by(STALE), None);
    assert_eq!(defaults.tick_limit(), TickLimit::from(TICK));
}
