use super::super::reading::Reading;
use crate::Measures;

#[derive(Clone, Copy, Debug)]
pub enum Status {
    Measured(fn(&Measures) -> Reading),
    ByLint(&'static str),
    NotImplemented,
}

impl Status {
    #[must_use]
    pub fn read(self, measures: &Measures) -> Reading {
        match self {
            Status::Measured(reader) => reader(measures),
            Status::ByLint(lint) => Reading::ByLint(lint),
            Status::NotImplemented => Reading::NotImplemented,
        }
    }
}
