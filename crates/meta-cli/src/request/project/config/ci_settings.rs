mod log_window;
mod millis;
mod program_name;
mod stop_limit;

use std::time::{Duration, Instant};

use serde::Deserialize;

use super::Config;
use crate::request::cli_failure::{CliFailure, Setup};
pub use log_window::LogWindow;
pub use millis::Millis;
pub use program_name::ProgramName;
use stop_limit::StopLimit;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
    appear_millis: Millis,
    remote_millis: Millis,
    git_millis: Millis,
    stop_millis: StopLimit,
    #[serde(flatten)]
    log_window: LogWindow,
}

impl CiSettings {
    pub const DEFAULTS: &'static str = include_str!("ci_settings/ci_defaults.toml");

    pub fn program(&self) -> &ProgramName {
        &self.program
    }

    pub fn poll_interval(&self) -> Millis {
        self.poll_millis
    }

    pub fn run_overdue(&self, waited: Duration) -> Option<()> {
        (waited >= self.appear_millis.duration()).then_some(())
    }

    pub fn remaining(&self, waited: Duration) -> Option<Duration> {
        self.timeout_millis.duration().checked_sub(waited).filter(|left| !left.is_zero())
    }

    pub fn remote_deadline(&self) -> Instant {
        Instant::now() + self.remote_millis.duration()
    }

    pub fn git_deadline(&self, from: Instant) -> Instant {
        from + self.git_millis.duration()
    }

    pub fn stop_limit(&self) -> StopLimit {
        self.stop_millis
    }

    pub fn log_window(&self) -> &LogWindow {
        &self.log_window
    }
}

impl TryFrom<&Config> for CiSettings {
    type Error = CliFailure;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let mut merged: toml::Table = toml::from_str(Self::DEFAULTS).unwrap_or_default();
        merged.extend(config.ci.clone());
        toml::Value::Table(merged)
            .try_into()
            .map_err(|failure| CliFailure::raised::<Setup>(format!("meta.toml [ci]: {failure}")))
    }
}

#[cfg(test)]
mod tests {
    use super::{CiSettings, Duration};
    use crate::request::project::config::Config;

    const CONFIGURED: &str = "[ci]\npoll_millis = 7";
    const STOP_LIMITS: [&str; 2] = ["[ci]\nstop_millis = 6999", "[ci]\nstop_millis = 7000"];
    const REFUSED: &str = "stop_millis must be under 7000 ms: Claude Code ends a Stop hook at 10 s";

    #[test]
    fn the_poll_interval_is_the_configured_poll_millis() {
        let config: Config = toml::from_str(CONFIGURED).unwrap();
        assert_eq!(CiSettings::try_from(&config).unwrap().poll_interval().duration(), Duration::from_millis(7));
    }

    #[test]
    fn a_stop_limit_at_seven_seconds_or_more_is_refused() {
        let configs = STOP_LIMITS.map(|text| toml::from_str::<Config>(text).unwrap());
        let [under, at] = configs.each_ref().map(CiSettings::try_from);
        assert!(under.is_ok_and(|settings| settings.stop_limit().deadline() > std::time::Instant::now()));
        assert!(at.unwrap_err().to_string().contains(REFUSED));
    }
}
