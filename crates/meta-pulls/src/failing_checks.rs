use std::fmt;

use crate::check_name::CheckName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FailingChecks(Vec<CheckName>);

impl FromIterator<CheckName> for FailingChecks {
    fn from_iter<Names: IntoIterator<Item = CheckName>>(names: Names) -> Self {
        Self(names.into_iter().collect())
    }
}

impl fmt::Display for FailingChecks {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        formatter.write_str(&names.join(", "))
    }
}
