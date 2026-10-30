use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellRules {
    forbidden: Vec<String>,
    wrappers: Vec<String>,
    exempt: Vec<String>,
}
