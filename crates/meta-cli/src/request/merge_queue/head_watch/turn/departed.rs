use meta_pulls::{Action, Departure, Stop, TransitionVisitor};
use meta_schema::PullRequest;

#[derive(Clone, Copy, Debug)]
pub struct Departed;

impl TransitionVisitor for Departed {
    type Output = Option<Departure>;

    fn act(self, _: Action) -> Self::Output {
        None
    }

    fn conflict(self, _: PullRequest) -> Self::Output {
        None
    }

    fn stop(self, _: Stop) -> Self::Output {
        None
    }

    fn left(self, departure: Departure) -> Self::Output {
        Some(departure)
    }
}
