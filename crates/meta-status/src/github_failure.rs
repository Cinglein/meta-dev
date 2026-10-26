use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GithubFailure(String);

impl GithubFailure {
    #[must_use]
    pub fn new(reason: impl fmt::Display) -> Self {
        Self(reason.to_string())
    }
}

impl fmt::Display for GithubFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
