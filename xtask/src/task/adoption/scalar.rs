use std::fmt;
use std::ops::Deref;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scalar(String);

impl From<String> for Scalar {
    fn from(text: String) -> Scalar {
        Scalar(text)
    }
}

impl Deref for Scalar {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
