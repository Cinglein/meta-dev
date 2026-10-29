use crate::action::Action;
use crate::departure::Departure;
use crate::detour::Detour;
use crate::stop::Stop;

pub trait TransitionVisitor {
    type Output;

    fn act(self, action: Action) -> Self::Output;
    fn detour(self, detour: Detour) -> Self::Output;
    fn stop(self, stop: Stop) -> Self::Output;
    fn left(self, departure: Departure) -> Self::Output;
}
