mod doctests;
mod target_kind;

use doctests::Doctests;
use serde::Deserialize;
use target_kind::TargetKind;

#[derive(Debug, Deserialize)]
pub struct BuiltTarget {
    doctest: Doctests,
    kind: Vec<TargetKind>,
}

impl BuiltTarget {
    pub fn doctested(&self) -> Option<&Doctests> {
        Some(&self.doctest).filter(|doctests| **doctests == Doctests::Run)
    }

    pub fn example(&self) -> Option<&TargetKind> {
        self.kind.iter().find(|kind| **kind == TargetKind::Example)
    }
}
