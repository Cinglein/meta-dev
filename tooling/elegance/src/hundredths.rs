use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hundredths(usize);

impl Hundredths {
    pub const WHOLE: usize = 100;

    #[must_use]
    pub fn new(hundredths: usize) -> Hundredths {
        Hundredths(hundredths)
    }
}

impl fmt::Display for Hundredths {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{:02}", self.0 / Self::WHOLE, self.0 % Self::WHOLE)
    }
}
