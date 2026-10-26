use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cause(String);

impl Cause {
    pub fn new(cause: impl fmt::Display) -> Self {
        Self(cause.to_string().lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or_default().to_owned())
    }
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
