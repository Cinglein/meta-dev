use std::fmt;

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

impl fmt::Display for BaseBranch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
