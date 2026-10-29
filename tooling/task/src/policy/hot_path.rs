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
            None => (path.ends_with(&self.0)).then(|| path.to_path_buf()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::HotPath;

    #[test]
    fn a_tree_is_hot_as_one_place_and_a_file_name_is_hot_wherever_it_is() {
        let tree = HotPath("goldens/snapshots/**".to_owned());
        let name = HotPath("lib.rs".to_owned());
        let hot = |hot: &HotPath, path: &str| hot.meeting(Path::new(path));
        assert_eq!(
            hot(&tree, "goldens/snapshots/a/b.snap"),
            Some(PathBuf::from("goldens/snapshots"))
        );
        assert_eq!(
            hot(&name, "crates/a/src/lib.rs"),
            Some(PathBuf::from("crates/a/src/lib.rs"))
        );
        assert_eq!(hot(&tree, "crates/a/src/lib.rs"), None);
    }
}
