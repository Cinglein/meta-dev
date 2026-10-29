use std::fmt;

use crate::action::Action;
use crate::departure::Departure;
use crate::detour::Detour;
use crate::stop::Stop;
use crate::transition_visitor::TransitionVisitor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transition {
    Act(Action),
    Detour(Detour),
    Stop(Stop),
    Left(Departure),
}

impl Transition {
    pub fn visit<Visitor: TransitionVisitor>(self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::Act(action) => visitor.act(action),
            Self::Detour(detour) => visitor.detour(detour),
            Self::Stop(stop) => visitor.stop(stop),
            Self::Left(departure) => visitor.left(departure),
        }
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Act(action) => action.fmt(formatter),
            Self::Detour(detour) => detour.fmt(formatter),
            Self::Stop(stop) => stop.fmt(formatter),
            Self::Left(departure) => departure.fmt(formatter),
        }
    }
}
