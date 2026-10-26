mod log_window;
mod millis;
mod program_name;

use serde::Deserialize;

pub use log_window::LogWindow;
pub use millis::Millis;
pub use program_name::ProgramName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
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

    pub fn poll_length(&self) -> usize {
        let polls = self.timeout_millis.duration().as_millis().checked_div(self.poll_millis.duration().as_millis());
        usize::try_from(polls.unwrap_or_default()).unwrap_or(usize::MAX)
    }

    pub fn log_window(&self) -> &LogWindow {
        &self.log_window
    }
}
