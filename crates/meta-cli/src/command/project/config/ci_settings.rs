mod line_count;
mod millis;
mod program_name;

use serde::Deserialize;

pub use line_count::LineCount;
pub use millis::Millis;
pub use program_name::ProgramName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
    log_lines: LineCount,
    log_context: LineCount,
}

impl CiSettings {
    pub const DEFAULTS: &'static str = include_str!("ci_settings/ci_defaults.toml");

    pub fn log_lines(&self) -> LineCount {
        self.log_lines
    }

    pub fn log_context(&self) -> LineCount {
        self.log_context
    }

    pub fn program(&self) -> &ProgramName {
        &self.program
    }

    pub fn poll_interval(&self) -> Millis {
        self.poll_millis
    }

    pub fn timeout(&self) -> Millis {
        self.timeout_millis
    }
}
