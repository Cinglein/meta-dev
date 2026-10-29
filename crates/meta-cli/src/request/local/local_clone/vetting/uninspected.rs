use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uninspected(usize);

impl Uninspected {
    pub fn counted(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for Uninspected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => Ok(()),
            1 => writeln!(formatter, "1 process could not be inspected"),
            count => writeln!(formatter, "{count} processes could not be inspected"),
        }
    }
}
