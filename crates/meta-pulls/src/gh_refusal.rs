use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhRefusal(String);

impl GhRefusal {
    #[must_use]
    pub fn new(reason: impl fmt::Display) -> Self {
        Self(reason.to_string().trim().to_owned())
    }
}

impl fmt::Display for GhRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
