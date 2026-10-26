use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Surface(PathBuf);

impl Surface {
    #[must_use]
    pub fn covers(&self, path: &Path) -> bool {
        path.starts_with(&self.0) || path.ends_with(&self.0)
    }
}

impl From<&str> for Surface {
    fn from(path: &str) -> Surface {
        Surface(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Surface;

    const MANIFEST: &str = "Cargo.toml";
    const CRATE_MANIFEST: &str = "crates/a/Cargo.toml";
    const SOURCE: &str = "crates/a/src/Cargo.rs";

    #[test]
    fn a_file_surface_covers_that_file_in_every_directory() {
        assert!(Surface::from(MANIFEST).covers(Path::new(CRATE_MANIFEST)));
        assert!(!Surface::from(MANIFEST).covers(Path::new(SOURCE)));
    }
}
