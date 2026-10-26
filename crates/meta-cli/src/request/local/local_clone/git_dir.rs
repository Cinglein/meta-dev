use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct GitDir(PathBuf);

impl GitDir {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for GitDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
