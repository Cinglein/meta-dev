mod package;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::crate_name::CrateName;
pub use package::Package;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    packages: Vec<Package>,
    workspace_root: PathBuf,
}

impl Metadata {
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn names_where(&self, kept: impl Fn(&Package) -> bool) -> BTreeSet<CrateName> {
        self.packages
            .iter()
            .filter(|package| kept(package))
            .map(|package| package.name().clone())
            .collect()
    }
}
