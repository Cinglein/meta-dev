use serde::Deserialize;

use super::program::Program;
use super::union::Union;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Programs(Vec<Program>);

impl Programs {
    pub fn names(&self, word: &str) -> bool {
        self.0.iter().any(|program| program.names(word))
    }
}

impl Union for Programs {
    fn union(self, other: Self) -> Self {
        Self([self.0, other.0].concat())
    }
}
