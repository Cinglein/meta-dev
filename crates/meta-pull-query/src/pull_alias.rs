use std::fmt;
use std::str::FromStr;

use meta_schema::PullRequest;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct PullAlias(PullRequest);

impl PullAlias {
    const PREFIX: &'static str = "pr";

    #[must_use]
    pub fn new(pull_request: PullRequest) -> Self {
        Self(pull_request)
    }
}

impl TryFrom<String> for PullAlias {
    type Error = <PullRequest as FromStr>::Err;

    fn try_from(alias: String) -> Result<Self, Self::Error> {
        alias.strip_prefix(Self::PREFIX).unwrap_or(&alias).parse().map(Self)
    }
}

impl fmt::Display for PullAlias {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", Self::PREFIX, self.0.number())
    }
}
