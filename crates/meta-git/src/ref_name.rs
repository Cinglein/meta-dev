use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefName(String);

impl RefName {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RefName {
    type Err = Infallible;

    fn from_str(name: &str) -> Result<RefName, Infallible> {
        Ok(RefName(name.to_owned()))
    }
}

impl fmt::Display for RefName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
