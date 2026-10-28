use std::fmt;
use std::process::Child;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leader(u32);

impl From<&Child> for Leader {
    fn from(child: &Child) -> Self {
        Self(child.id())
    }
}

impl fmt::Display for Leader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
