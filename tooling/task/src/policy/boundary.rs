use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Boundary(Vec<PathBuf>);

impl Boundary {
    #[must_use]
    pub fn covers(&self, path: &Path) -> bool {
        self.0.iter().any(|boundary| path.starts_with(boundary))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::Boundary;

    const BOUNDARY: &str = "crates/board/src/bits";
    const INSIDE: &str = "crates/board/src/bits/mask.rs";
    const OUTSIDE: &str = "crates/board/src/moves.rs";

    #[test]
    fn a_boundary_covers_the_files_below_it_only() {
        let boundary = Boundary(vec![PathBuf::from(BOUNDARY)]);
        assert!(boundary.covers(Path::new(INSIDE)) && !boundary.covers(Path::new(OUTSIDE)));
    }
}
