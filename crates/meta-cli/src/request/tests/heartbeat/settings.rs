use std::time::Duration;

use super::{SETTINGS, settings};

const OVERLONG_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 300\nstale_seconds = 900";
const SHORT_STALE: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 300";
const ZERO_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 0\nstale_seconds = 900";
const UNKNOWN_KEY: &str = "[heartbeat]\ninterval = 300";
const TICK_SECONDS: u64 = 60;
const STALE_SECONDS: u64 = 900;

#[test]
fn settings_read_the_configured_limits() {
    let valid = settings(SETTINGS).unwrap();
    assert_eq!(valid.interval().to_string(), "300");
    assert_eq!(valid.tick_limit(), Duration::from_secs(TICK_SECONDS));
    assert_eq!(valid.stale_limit(), Duration::from_secs(STALE_SECONDS));
}

#[test]
fn settings_need_a_tick_shorter_than_the_interval_which_is_shorter_than_the_stale_limit() {
    let refused = [OVERLONG_TICK, SHORT_STALE, ZERO_TICK, UNKNOWN_KEY].map(|text| settings(text).is_err());
    assert_eq!(refused, [true; 4]);
}

#[test]
fn absent_settings_take_the_defaults() {
    let defaults = settings("").unwrap();
    assert_eq!(defaults.interval().to_string(), "300");
}
