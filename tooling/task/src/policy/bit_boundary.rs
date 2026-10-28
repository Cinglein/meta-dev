use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BitBoundary(Vec<PathBuf>);

impl BitBoundary {
    #[must_use]
    pub fn covers(&self, path: &Path) -> bool {
        self.0.iter().any(|boundary| path.starts_with(boundary))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::BitBoundary;

    const BOUNDARY: &str = "crates/board/src/bits";
    const INSIDE: &str = "crates/board/src/bits/mask.rs";
    const OUTSIDE: &str = "crates/board/src/moves.rs";

    #[test]
    fn a_boundary_covers_the_files_below_it_only() {
        let boundary = BitBoundary(vec![PathBuf::from(BOUNDARY)]);
        assert!(boundary.covers(Path::new(INSIDE)) && !boundary.covers(Path::new(OUTSIDE)));
    }
}
