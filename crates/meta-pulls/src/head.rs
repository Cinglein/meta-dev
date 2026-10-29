mod behind;
mod checking;
mod conflicted;
mod gone;
mod green;
mod red;
mod unmergeable;

use std::fmt;

use crate::patience::Patience;
use crate::transition::Transition;
pub use behind::Behind;
pub use checking::Checking;
pub use conflicted::Conflicted;
pub use gone::Gone;
pub use green::Green;
pub use red::Red;
pub use unmergeable::Unmergeable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Head {
    Gone(Gone),
    Conflicted(Conflicted),
    Red(Red),
    Behind(Behind),
    Green(Green),
    Unmergeable(Unmergeable),
    Checking(Checking),
}

impl Head {
    #[must_use]
    pub fn step(self, patience: Patience) -> Transition {
        match self {
            Self::Gone(gone) => gone.left(),
            Self::Conflicted(conflicted) => conflicted.detoured(),
            Self::Red(red) => red.stopped(),
            Self::Behind(behind) => behind.caught_up(),
            Self::Green(green) => green.merged(),
            Self::Unmergeable(unmergeable) => unmergeable.stopped(),
            Self::Checking(checking) => checking.waited(patience),
        }
    }
}

impl fmt::Display for Head {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gone(_) => formatter.write_str("gone"),
            Self::Conflicted(_) => formatter.write_str("conflicted"),
            Self::Red(_) => formatter.write_str("red"),
            Self::Behind(_) => formatter.write_str("behind"),
            Self::Green(_) => formatter.write_str("green"),
            Self::Unmergeable(_) => formatter.write_str("unmergeable"),
            Self::Checking(_) => formatter.write_str("checking"),
        }
    }
}
