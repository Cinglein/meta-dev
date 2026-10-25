mod millis;
mod program_name;

use serde::Deserialize;

pub use millis::Millis;
pub use program_name::ProgramName;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CiSettings {
    program: ProgramName,
    poll_millis: Millis,
    timeout_millis: Millis,
}

impl CiSettings {
    const DEFAULTS: &'static str = include_str!("ci_settings/ci_defaults.toml");

    pub fn embedded() -> Self {
        toml::from_str(Self::DEFAULTS).unwrap_or_default()
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
