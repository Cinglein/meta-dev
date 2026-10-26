use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::IgnoredAny;

#[derive(Debug, Deserialize)]
pub struct MetadataPackage {
    manifest_path: PathBuf,
    source: Option<IgnoredAny>,
}

impl MetadataPackage {
    pub fn local_manifest(&self) -> Option<&Path> {
        self.source.is_none().then_some(self.manifest_path.as_path())
    }
}
