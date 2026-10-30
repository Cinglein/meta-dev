use std::fmt;

use meta_schema::PullRequest;

use crate::action::Action;
use crate::departure::Departure;
use crate::stop::Stop;
use crate::transition_visitor::TransitionVisitor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transition {
    Act(Action),
    Conflict(PullRequest),
    Stop(Stop),
    Left(Departure),
}

impl Transition {
    pub fn visit<Visitor: TransitionVisitor>(self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::Act(action) => visitor.act(action),
            Self::Conflict(pull_request) => visitor.conflict(pull_request),
            Self::Stop(stop) => visitor.stop(stop),
            Self::Left(departure) => visitor.left(departure),
        }
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Act(action) => action.fmt(formatter),
            Self::Conflict(pull_request) => write!(formatter, "{pull_request} conflicts with main: needs conflict fix"),
            Self::Stop(stop) => stop.fmt(formatter),
            Self::Left(departure) => departure.fmt(formatter),
        }
    }
}
