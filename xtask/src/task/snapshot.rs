use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Entry {
    Directory(PathBuf),
    File(PathBuf, Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Snapshot {
    entries: Vec<Entry>,
}

impl Snapshot {
    const GIT: &str = ".git";

    pub fn files(root: &Path) -> Snapshot {
        Snapshot { entries: Self::walk(root, root, true) }
    }

    pub fn repository(root: &Path) -> Snapshot {
        let git = root.join(Self::GIT);
        Snapshot { entries: Self::walk(&git, &git, false) }
    }

    pub fn restore(&self, root: &Path) {
        let git = root.join(Self::GIT);
        self.entries.iter().for_each(|entry| match entry {
            Entry::Directory(path) => fs::create_dir_all(git.join(path)).expect("restore directory"),
            Entry::File(path, bytes) => fs::write(git.join(path), bytes).expect("restore file"),
        });
    }

    fn walk(root: &Path, directory: &Path, skip_git: bool) -> Vec<Entry> {
        let mut listed: Vec<PathBuf> = fs::read_dir(directory)
            .expect("read directory")
            .map(|found| found.expect("directory entry").path())
            .filter(|path| !(skip_git && path.file_name().is_some_and(|name| name == Self::GIT)))
            .collect();
        listed.sort();
        listed
            .into_iter()
            .flat_map(|path| {
                let relative = path.strip_prefix(root).expect("under root").to_path_buf();
                if path.is_dir() {
                    let mut inside = vec![Entry::Directory(relative)];
                    inside.extend(Self::walk(root, &path, false));
                    inside
                } else {
                    vec![Entry::File(relative, fs::read(&path).expect("read file"))]
                }
            })
            .collect()
    }
}
