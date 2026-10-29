use std::ops::ControlFlow;

use meta_schema::Record;

use super::attempt::Attempt;
use super::pacing::Pacing;
use meta_run::Budget;

#[derive(Debug)]
pub enum Outcome {
    Appended(Record),
    Raced,
}

impl Outcome {
    pub fn step<Failure>(
        result: Result<Self, Failure>,
        attempt: Attempt,
        pacing: Pacing,
        budget: &Budget,
    ) -> ControlFlow<Result<Record, Failure>> {
        match result {
            Ok(Self::Appended(record)) => ControlFlow::Break(Ok(record)),
            Ok(Self::Raced) => {
                budget.rest(pacing.pause(attempt));
                ControlFlow::Continue(())
            }
            Err(failure) => ControlFlow::Break(Err(failure)),
        }
    }
}
