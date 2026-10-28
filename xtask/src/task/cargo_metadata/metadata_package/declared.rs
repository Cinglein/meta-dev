mod dependency_kind;

use serde::Deserialize;
use task::CrateName;

use dependency_kind::DependencyKind;

#[derive(Debug, Deserialize)]
pub struct Declared {
    name: CrateName,
    rename: Option<CrateName>,
    kind: Option<DependencyKind>,
    features: Vec<String>,
}

impl Declared {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn rename(&self) -> Option<&CrateName> {
        self.rename.as_ref()
    }

    pub fn features(&self) -> &[String] {
        &self.features
    }

    pub fn built(&self) -> bool {
        self.kind != Some(DependencyKind::Dev)
    }
}
