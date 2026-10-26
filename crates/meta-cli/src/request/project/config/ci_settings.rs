mod log_window;
mod millis;
mod program_name;

use std::time::Duration;

use serde::Deserialize;

pub use log_window::LogWindow;
pub use millis::Millis;
pub use program_name::ProgramName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
    appear_millis: Millis,
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

    pub fn log_window(&self) -> &LogWindow {
        &self.log_window
    }
}
