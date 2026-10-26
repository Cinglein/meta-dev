use std::fmt;
use std::str::FromStr;

use crate::request::cli_failure::{CliFailure, Misuse};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionName {
    Source,
    Tag,
    Addresses,
    From,
    All,
}

impl FromStr for OptionName {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "--source" => Ok(Self::Source),
            "--tag" => Ok(Self::Tag),
            "--addresses" => Ok(Self::Addresses),
            "--from" => Ok(Self::From),
            "--all" => Ok(Self::All),
            other => Err(CliFailure::raised::<Misuse>(format!(
                "unknown option {other}; put text starting with -- after a bare --"
            ))),
        }
    }
}

impl fmt::Display for OptionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Source => "--source",
            Self::Tag => "--tag",
            Self::Addresses => "--addresses",
            Self::From => "--from",
            Self::All => "--all",
        };
        formatter.write_str(name)
    }
}
