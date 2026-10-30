mod cargo_line;
mod message;

use std::convert::Infallible;
use std::str::FromStr;

use task::{Failure, FailureKind, FailureSet, Printed};

use cargo_line::CargoLine;
use message::{Message, Standing};

#[derive(Debug)]
pub(super) struct Diagnostics {
    placed: Vec<Message>,
    unplaced: Vec<Message>,
}

impl Diagnostics {
    const UNPLACED: &str = "clippy";

    pub(super) fn reading(printed: &str) -> Option<Failure> {
        let Ok(diagnostics) = printed.parse::<Diagnostics>();
        Some(diagnostics).filter(|diagnostics| diagnostics.messages().next().is_some()).map(|diagnostics| {
            Failure::Found { printed: Printed::Text(diagnostics.rendered()), found: diagnostics.found() }
        })
    }

    fn messages(&self) -> impl Iterator<Item = &Message> {
        self.placed.iter().chain(&self.unplaced)
    }

    fn rendered(&self) -> String {
        self.messages().filter_map(Message::rendered).collect()
    }

    fn found(&self) -> FailureSet {
        let infra = (!self.unplaced.is_empty()).then(|| FailureKind::Infra { job: Self::UNPLACED.into() });
        self.placed.iter().map(Message::failure).chain(infra).fold(FailureSet::default(), FailureSet::recording)
    }
}

impl FromStr for Diagnostics {
    type Err = Infallible;

    fn from_str(printed: &str) -> Result<Diagnostics, Infallible> {
        let mut diagnostics = Diagnostics { placed: Vec::new(), unplaced: Vec::new() };
        printed
            .lines()
            .filter_map(|line| line.parse::<CargoLine>().ok())
            .filter_map(CargoLine::compiler_message)
            .for_each(|message| match message.standing() {
                Some(Standing::Placed) => diagnostics.placed.push(message),
                Some(Standing::Unplaced) => diagnostics.unplaced.push(message),
                Some(Standing::Summary) | None => {}
            });
        Ok(diagnostics)
    }
}
