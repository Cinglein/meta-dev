use derive_more::{Add, AddAssign, Display, From, Into};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Display, From, Into,
)]
pub struct MutantCount(usize);
