use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::unleased::Unleased;
use crate::request::output::Listed;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownProjects {
    roots: Vec<PathBuf>,
}

impl KnownProjects {
    const FILE: &'static str = "meta/projects.toml";
    const CONFIG: &'static str = ".config";

    pub fn file() -> PathBuf {
        let config = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
        let home = || std::env::var_os("HOME").map(|home| PathBuf::from(home).join(Self::CONFIG));
        config.or_else(home).unwrap_or_default().join(Self::FILE)
    }

    pub fn read(file: &Path) -> Result<Self, Unleased> {
        let unread = |reason: Listed| Unleased::Unregistered(file.to_path_buf(), reason);
        let text = std::fs::read_to_string(file).map_err(|failure| unread(Listed::new(failure)))?;
        toml::from_str(&text).map_err(|failure| unread(Listed::new(failure)))
    }

    pub fn admits(&self, root: &Path) -> Option<()> {
        let root = root.canonicalize().ok()?;
        self.roots.iter().filter_map(|known| known.canonicalize().ok()).find(|known| *known == root).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::KnownProjects;

    const FILE: &str = "projects.toml";
    const LISTED: &str = "listed";
    const UNLISTED: &str = "unlisted";
    const ROOTS: &str = "roots";

    #[test]
    fn only_a_listed_root_is_a_known_project_and_an_unreadable_file_admits_none() {
        let dir = tempfile::tempdir().unwrap();
        let [listed, unlisted] = [LISTED, UNLISTED].map(|name| dir.path().join(name));
        std::fs::create_dir(&listed).and_then(|()| std::fs::create_dir(&unlisted)).unwrap();
        let quoted = serde_json::to_string(&listed).unwrap();
        std::fs::write(dir.path().join(FILE), format!("{ROOTS} = [{quoted}]\n")).unwrap();
        let known = KnownProjects::read(&dir.path().join(FILE)).unwrap();
        assert_eq!([known.admits(&listed), known.admits(&unlisted)], [Some(()), None]);
        assert!(KnownProjects::read(&dir.path().join(UNLISTED).join(FILE)).is_err());
    }
}
