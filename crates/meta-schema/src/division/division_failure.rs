use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DivisionFailure {
    #[error(
        "division {0}: begins features-and-maintenance, performance-and-cost or architecture-and-planning; no empty part"
    )]
    Unplaced(Unparsed),
}
