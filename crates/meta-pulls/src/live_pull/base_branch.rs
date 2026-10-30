use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

use super::RuleSource;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct BaseBranch(String);

impl BaseBranch {
    #[must_use]
    pub fn endpoint(&self, source: RuleSource) -> String {
        match source {
            RuleSource::Rulesets => format!("repos/{{owner}}/{{repo}}/rules/branches/{self}"),
            RuleSource::Protection => format!("repos/{{owner}}/{{repo}}/branches/{self}"),
        }
    }
}

impl FromStr for BaseBranch {
    type Err = Infallible;

    fn from_str(printed: &str) -> Result<Self, Self::Err> {
        Ok(Self(printed.trim().to_owned()))
    }
}

impl fmt::Display for BaseBranch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
