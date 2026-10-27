mod doctests;

use doctests::Doctests;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BuiltTarget {
    doctest: Doctests,
}

impl BuiltTarget {
    pub fn doctested(&self) -> Option<&Doctests> {
        Some(&self.doctest).filter(|doctests| **doctests == Doctests::Run)
    }
}
