use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkflowName(String);

impl WorkflowName {
    const REQUIRED: [&'static str; 2] = ["ci", "mutants"];

    #[must_use]
    pub fn required() -> Vec<Self> {
        Self::REQUIRED.iter().map(|name| Self((*name).to_owned())).collect()
    }
}

impl fmt::Display for WorkflowName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
