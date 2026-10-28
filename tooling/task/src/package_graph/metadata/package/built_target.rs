mod target_kind;

use serde::Deserialize;
use target_kind::TargetKind;

#[derive(Debug, Deserialize)]
pub struct BuiltTarget {
    kind: Vec<TargetKind>,
}

impl BuiltTarget {
    pub fn example(&self) -> Option<&TargetKind> {
        self.kind.iter().find(|kind| **kind == TargetKind::Example)
    }
}
