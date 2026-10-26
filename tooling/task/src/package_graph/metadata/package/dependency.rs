use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
    path: Option<String>,
}

impl Dependency {
    pub fn local_name(&self) -> Option<&CrateName> {
        self.path.as_ref().map(|_| &self.name)
    }
}
