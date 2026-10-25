use derive_more::{Add, Display, From, Into, Sum};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Add, Sum, Display, From, Into,
)]
pub struct ReexportCount(usize);
