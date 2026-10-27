use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sha(String);

impl Sha {
    const HEX_LENGTH_PER_BYTE: usize = 2;

    pub(crate) fn digest_length(&self) -> usize {
        self.0.len() / Self::HEX_LENGTH_PER_BYTE
    }
}

impl FromStr for Sha {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.trim().to_owned()))
    }
}

impl fmt::Display for Sha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
