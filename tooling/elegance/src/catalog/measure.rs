mod recall;
mod score;
mod status;

use std::collections::BTreeSet;
use std::fmt;

use super::direction::Direction;
use super::measure_graph::MeasureGraph;
use super::measure_id::MeasureId;
use super::reading::Reading;
use crate::Measures;
use crate::smell_inventory::SmellInventory;
use crate::source_path::SourcePath;
use recall::Recall;
use score::Score;
use status::Status;

#[derive(Clone, Copy, Debug)]
pub struct Measure {
    id: MeasureId,
    name: &'static str,
    graph: MeasureGraph,
    direction: Direction,
    status: Status,
    score: Option<Score>,
}

impl Measure {
    #[must_use]
    pub const fn new(id: MeasureId, name: &'static str, graph: MeasureGraph, direction: Direction) -> Measure {
        Measure { id, name, graph, direction, status: Status::NotImplemented, score: None }
    }

    #[must_use]
    pub const fn measured(self, reader: fn(&Measures) -> Reading) -> Measure {
        Measure { status: Status::Measured(reader), ..self }
    }

    #[must_use]
    pub const fn by_lint(self, lint: &'static str) -> Measure {
        Measure { status: Status::ByLint(lint), ..self }
    }

    #[must_use]
    pub const fn scored(self, tag: &'static str, located: fn(&Measures) -> Vec<BTreeSet<SourcePath>>) -> Measure {
        Measure { score: Some(Score::new(tag, located)), ..self }
    }

    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    #[must_use]
    pub fn recall(&self, measures: &Measures, inventory: &SmellInventory) -> Option<Recall> {
        self.score.map(|score| score.recall(measures, inventory))
    }
}

impl fmt::Display for Measure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} [{}, {}]", self.id, self.name, self.graph, self.direction)
    }
}
