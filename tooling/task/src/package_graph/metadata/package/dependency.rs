mod dependency_kind;

use serde::Deserialize;

use crate::crate_name::CrateName;
use dependency_kind::DependencyKind;

#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
    path: Option<String>,
    #[serde(default)]
    kind: Option<DependencyKind>,
}

impl Dependency {
    #[must_use]
    pub fn local_name(&self) -> Option<&CrateName> {
        self.path.as_ref().map(|_| &self.name)
    }

    #[must_use]
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    #[must_use]
    pub fn for_tests(&self) -> bool {
        self.kind == Some(DependencyKind::Dev)
    }
}
