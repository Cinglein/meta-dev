use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BranchName(String);

impl FromStr for BranchName {
    type Err = Infallible;

    fn from_str(name: &str) -> Result<BranchName, Infallible> {
        Ok(BranchName(name.to_owned()))
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::BranchName;
    const BASE: &str = "origin/main";

    #[test]
    fn a_branch_name_prints_as_it_was_spelled() {
        let Ok(name) = BASE.parse::<BranchName>();
        assert_eq!(name.to_string(), BASE);
    }
}
