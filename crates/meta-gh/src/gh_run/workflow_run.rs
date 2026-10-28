use serde::Deserialize;

use crate::database_id::DatabaseId;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRun {
    #[serde(default, rename = "databaseId")]
    database_id: DatabaseId,
}

impl WorkflowRun {
    pub fn database_id(self) -> DatabaseId {
        self.database_id
    }
}
