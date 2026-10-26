use std::ops::ControlFlow;

use meta_schema::Record;

use super::attempt::Attempt;

#[derive(Debug)]
pub enum Outcome {
    Appended(Record),
    Raced,
}

impl Outcome {
    pub fn step<Failure>(result: Result<Self, Failure>, attempt: Attempt) -> ControlFlow<Result<Record, Failure>> {
        match result {
            Ok(Self::Appended(record)) => ControlFlow::Break(Ok(record)),
            Ok(Self::Raced) => {
                std::thread::sleep(attempt.pause());
                ControlFlow::Continue(())
            }
            Err(failure) => ControlFlow::Break(Err(failure)),
        }
    }
}
