use std::str::FromStr;

use crate::request::cli_failure::{CliFailure, Misuse};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceView {
    Last,
}

impl FromStr for TraceView {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "last" => Ok(Self::Last),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown trace command `{other}`"))),
        }
    }
}
