use std::fmt;
use std::process::Output;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detail(String);

impl From<&Output> for Detail {
    fn from(output: &Output) -> Self {
        Self(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

impl FromStr for Detail {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.to_owned()))
    }
}

impl fmt::Display for Detail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Detail {
    pub fn from_reason(reason: impl ToString) -> Self {
        Self(reason.to_string())
    }
}
