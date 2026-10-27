use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct SparedName(String);

impl SparedName {
    const FLOOR: [&'static str; 2] = ["claude", "Claude.app"];

    pub fn floor() -> Vec<Self> {
        Self::FLOOR.iter().map(|name| Self((*name).to_owned())).collect()
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}
