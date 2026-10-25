use derive_more::{Add, AddAssign, From, Into, SubAssign};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Add,
    AddAssign,
    SubAssign,
    From,
    Into,
)]
pub struct NestingDepth(usize);

impl NestingDepth {
    pub const LEVEL: NestingDepth = NestingDepth(1);

    #[must_use]
    pub const fn is_surface(self) -> bool {
        self.0 == 0
    }
}
