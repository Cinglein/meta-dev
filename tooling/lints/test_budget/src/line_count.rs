use derive_more::{Add, Display, From, Into};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Add, Display, From, Into)]
pub struct LineCount(usize);
