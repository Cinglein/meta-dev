use std::time::Duration;

use meta_golden::Golden;

use super::{Beat, SETTINGS};

const ABSENT: &str = "";
const OVERLONG_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 300\nstale_seconds = 900";
const SHORT_STALE: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 300";
const ZERO_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 0\nstale_seconds = 900";
const UNKNOWN_KEY: &str = "[heartbeat]\ninterval = 300";
const STALE: Duration = Duration::from_mins(15);
const WITHIN: Duration = Duration::from_mins(14);

#[test]
fn settings_read_the_configured_limits_take_the_defaults_when_absent_and_need_a_tick_shorter_than_the_interval() {
    let read = [SETTINGS, ABSENT, OVERLONG_TICK, SHORT_STALE, ZERO_TICK, UNKNOWN_KEY].map(Beat::settings);
    Golden::for_test(module_path!(), "settings").debugged(&read).unwrap();
    let valid = Beat::settings(SETTINGS).unwrap();
    assert_eq!([STALE, STALE + WITHIN].map(|age| valid.stale_limit().passed_by(age)), [None, Some(STALE)]);
}
