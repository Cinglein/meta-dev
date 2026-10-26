use std::fmt;
use std::process::Output;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stderr(String);

impl From<&Output> for Stderr {
    fn from(output: &Output) -> Self {
        Self(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

impl fmt::Display for Stderr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
