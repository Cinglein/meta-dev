use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unparsed(String);

impl From<&str> for Unparsed {
    fn from(text: &str) -> Self {
        Self(text.to_owned())
    }
}

impl fmt::Display for Unparsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
