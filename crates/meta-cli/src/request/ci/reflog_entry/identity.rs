use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity(String);

impl FromStr for Identity {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.to_owned()))
    }
}
