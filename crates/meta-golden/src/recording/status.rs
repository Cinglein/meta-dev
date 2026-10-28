use std::fmt;
use std::process::ExitCode;
use std::str::FromStr;

use meta_run::Ran;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status(i32);

impl Status {
    const SIGNALLED: i32 = 128;

    pub fn exit_code(self) -> ExitCode {
        u8::try_from(self.0).map_or(ExitCode::FAILURE, ExitCode::from)
    }
}

impl From<&Ran> for Status {
    fn from(ran: &Ran) -> Self {
        Self(ran.status().code().unwrap_or(Self::SIGNALLED))
    }
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl FromStr for Status {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse().map(Self)
    }
}
