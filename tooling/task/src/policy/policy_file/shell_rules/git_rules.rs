use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRules {
    forbidden: Vec<String>,
    nested: BTreeMap<String, Vec<String>>,
}
