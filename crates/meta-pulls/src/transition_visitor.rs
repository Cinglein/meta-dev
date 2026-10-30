use meta_schema::PullRequest;

use crate::action::Action;
use crate::departure::Departure;
use crate::stop::Stop;

pub trait TransitionVisitor {
    type Output;

    fn act(self, action: Action) -> Self::Output;
    fn conflict(self, pull_request: PullRequest) -> Self::Output;
    fn stop(self, stop: Stop) -> Self::Output;
    fn left(self, departure: Departure) -> Self::Output;
}
