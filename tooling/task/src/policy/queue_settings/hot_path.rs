use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HotPath(String);

impl HotPath {
    const TREE: &str = "/**";

    #[must_use]
    pub fn meeting(&self, path: &Path) -> Option<PathBuf> {
        match self.0.strip_suffix(Self::TREE) {
            Some(tree) => path.starts_with(tree).then(|| PathBuf::from(tree)),
            None => path.ends_with(&self.0).then(|| path.to_path_buf()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::HotPath;

    const TREE: &str = "goldens/snapshots/**";
    const NAME: &str = "lib.rs";
    const SNAPSHOT: &str = "goldens/snapshots/a/b.snap";
    const LIB: &str = "crates/a/src/lib.rs";

    #[test]
    fn a_tree_is_hot_as_one_place_and_a_file_name_is_hot_wherever_it_is() {
        let hot = |pattern: &str, path: &str| HotPath(pattern.to_owned()).meeting(Path::new(path));
        let met = [hot(TREE, SNAPSHOT), hot(NAME, LIB), hot(TREE, LIB)];
        let places = [Some("goldens/snapshots"), Some(LIB), None];
        assert_eq!(
            met.map(|path| path.map(|path| path.display().to_string())),
            places.map(|place| place.map(str::to_owned))
        );
    }
}
