use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct SparedName(String);

impl SparedName {
    pub fn text(&self) -> &str {
        &self.0
    }
}
