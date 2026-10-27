use meta_schema::{LevelName, Text};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    name: LevelName,
    meaning: Text,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<LevelName>,
}

impl Level {
    #[must_use]
    pub fn top(name: LevelName, meaning: Text) -> Level {
        Level { name, meaning, parent: None }
    }

    #[must_use]
    pub fn name(&self) -> &LevelName {
        &self.name
    }

    #[must_use]
    pub fn parent(&self) -> Option<&LevelName> {
        self.parent.as_ref()
    }
}
