mod status;

use std::fmt;

use super::direction::Direction;
use super::measure_graph::MeasureGraph;
use super::measure_id::MeasureId;
use super::reading::Reading;
use crate::Measures;
use status::Status;

#[derive(Clone, Copy, Debug)]
pub struct Measure {
    id: MeasureId,
    name: &'static str,
    graph: MeasureGraph,
    direction: Direction,
    status: Status,
}

impl Measure {
    #[must_use]
    pub const fn new(id: MeasureId, name: &'static str, graph: MeasureGraph, direction: Direction) -> Measure {
        Measure { id, name, graph, direction, status: Status::NotImplemented }
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
    pub fn status(&self) -> Status {
        self.status
    }
}

impl fmt::Display for Measure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} [{}, {}]", self.id, self.name, self.graph, self.direction)
    }
}
