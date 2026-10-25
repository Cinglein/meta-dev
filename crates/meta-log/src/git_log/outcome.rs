use std::ops::ControlFlow;

use meta_schema::Record;

use super::attempt::Attempt;
use crate::log_failure::LogFailure;

#[derive(Debug)]
pub enum Outcome {
    Appended(Record),
    Raced,
}

impl Outcome {
    pub fn step(result: Result<Self, LogFailure>, attempt: Attempt) -> ControlFlow<Result<Record, LogFailure>> {
        match result {
            Ok(Self::Appended(record)) => ControlFlow::Break(Ok(record)),
            Ok(Self::Raced) => {
                attempt.pause();
                ControlFlow::Continue(())
            }
            Err(failure) => ControlFlow::Break(Err(failure)),
        }
    }
}
