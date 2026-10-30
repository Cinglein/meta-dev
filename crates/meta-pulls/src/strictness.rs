use std::convert::Infallible;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strictness {
    Strict,
    Loose,
}

impl Strictness {
    const STRICT: &'static str = "true";

    #[must_use]
    pub fn loose(self) -> Option<()> {
        match self {
            Self::Loose => Some(()),
            Self::Strict => None,
        }
    }
}

impl FromStr for Strictness {
    type Err = Infallible;

    fn from_str(printed: &str) -> Result<Self, Self::Err> {
        Ok(if printed.trim() == Self::STRICT { Self::Strict } else { Self::Loose })
    }
}
