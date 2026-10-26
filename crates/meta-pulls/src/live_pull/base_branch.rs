use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct BaseBranch(String);

impl BaseBranch {
    #[must_use]
    pub fn rules_endpoint(&self) -> String {
        format!("repos/{{owner}}/{{repo}}/rules/branches/{self}")
    }
}

impl fmt::Display for BaseBranch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
