use std::fmt;

use meta_listing::{Kind, Listing, Scope};
use meta_log::LogStore;
use meta_state::{State, Summary};

use crate::request::cli_failure::CliFailure;

#[derive(Debug)]
pub struct Listed(String);

impl Listed {
    pub fn new(text: impl fmt::Display) -> Self {
        Self(text.to_string())
    }

    pub fn from_log(log: &impl LogStore, kind: Option<Kind>, scope: Scope) -> Result<Self, CliFailure> {
        let snapshot = log.read()?;
        let state = State::fold(snapshot.records());
        Ok(kind.map_or_else(
            || Self::new(format!("{}{snapshot}", Summary::new(&state))),
            |kind| Self::new(format!("{}{snapshot}", Listing::new(&state, kind, scope))),
        ))
    }
}

impl fmt::Display for Listed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
