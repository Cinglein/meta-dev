mod git_limit;
mod log_window;
mod millis;
mod program_name;

use std::time::{Duration, Instant};

use serde::Deserialize;

pub use git_limit::GitLimit;
pub use log_window::LogWindow;
pub use millis::Millis;
pub use program_name::ProgramName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
    appear_millis: Millis,
    remote_millis: Millis,
    git_millis: GitLimit,
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

    pub fn git_limit(&self) -> GitLimit {
        self.git_millis
    }

    pub fn log_window(&self) -> &LogWindow {
        &self.log_window
    }
}

#[cfg(test)]
mod tests {
    use super::Duration;
    use crate::request::project::config::Config;

    const CONFIGURED: &str = "[ci]\npoll_millis = 7";

    #[test]
    fn the_poll_interval_is_the_configured_poll_millis() {
        let config: Config = toml::from_str(CONFIGURED).unwrap();
        assert_eq!(config.ci_settings().poll_interval().duration(), Duration::from_millis(7));
    }
}
