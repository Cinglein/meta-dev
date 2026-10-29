mod cargo_line;
mod message;

use std::convert::Infallible;
use std::str::FromStr;

use task::{Failure, FailureSet};

use cargo_line::CargoLine;
use message::Message;

#[derive(Debug)]
pub(super) struct Diagnostics {
    errors: Vec<Message>,
}

impl Diagnostics {
    pub(super) fn reading(printed: &str) -> Option<Failure> {
        let Ok(diagnostics) = printed.parse::<Diagnostics>();
        Some(diagnostics).filter(|diagnostics| !diagnostics.errors.is_empty()).map(|diagnostics| Failure::Found {
            printed: diagnostics.errors.iter().filter_map(Message::rendered).collect(),
            found: diagnostics.found(),
        })
    }

    pub(super) fn found(&self) -> FailureSet {
        self.errors.iter().fold(FailureSet::default(), |set, message| set.recording(message.failure()))
    }
}

impl FromStr for Diagnostics {
    type Err = Infallible;

    fn from_str(printed: &str) -> Result<Diagnostics, Infallible> {
        Ok(Diagnostics {
            errors: printed
                .lines()
                .filter_map(|line| line.parse::<CargoLine>().ok())
                .filter_map(CargoLine::error)
                .collect(),
        })
    }
}
