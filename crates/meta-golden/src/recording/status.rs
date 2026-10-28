use std::process::ExitCode;

use meta_run::Ran;
use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Status {
    Succeeded,
    Failed,
}

impl Status {
    pub fn exit_code(self) -> ExitCode {
        match self {
            Self::Succeeded => ExitCode::SUCCESS,
            Self::Failed => ExitCode::FAILURE,
        }
    }
}

impl From<&Ran> for Status {
    fn from(ran: &Ran) -> Self {
        if ran.success() { Self::Succeeded } else { Self::Failed }
    }
}
