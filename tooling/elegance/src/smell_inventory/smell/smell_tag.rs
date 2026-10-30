use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SmellTag(String);

impl PartialEq<str> for SmellTag {
    fn eq(&self, tag: &str) -> bool {
        self.0 == tag
    }
}
