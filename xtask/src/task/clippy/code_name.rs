use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(super) struct CodeName(String);

impl CodeName {
    pub(super) fn is_lint(&self) -> bool {
        self.0.chars().any(char::is_lowercase)
    }
}
