use std::num::NonZeroUsize;

use derive_more::Display;

use super::available_gib::AvailableGib;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub struct CompilerTasks(NonZeroUsize);

impl CompilerTasks {
    #[must_use]
    pub fn within(budget: AvailableGib, each: AvailableGib) -> CompilerTasks {
        CompilerTasks(NonZeroUsize::new(budget.divided_by(each)).unwrap_or(NonZeroUsize::MIN))
    }
}
