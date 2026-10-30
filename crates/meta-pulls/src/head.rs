mod barred;
mod behind;
mod checking;
mod conflicted;
mod gone;
mod green;
mod red;
mod unmergeable;

use crate::patience::Patience;
use crate::transition::Transition;
pub use barred::Barred;
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
    Barred(Barred),
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
            Self::Barred(barred) => barred.stopped(),
            Self::Conflicted(conflicted) => conflicted.skipped(),
            Self::Red(red) => red.stopped(),
            Self::Behind(behind) => behind.caught_up(),
            Self::Green(green) => green.merged(),
            Self::Unmergeable(unmergeable) => unmergeable.stopped(),
            Self::Checking(checking) => checking.waited(patience),
        }
    }
}
