use meta_schema::ToolName;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModalTools {
    modal: Vec<ToolName>,
}
