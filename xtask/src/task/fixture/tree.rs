use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

static BUILT: LazyLock<Mutex<HashMap<Tree, Arc<OnceLock<Tree>>>>> = LazyLock::new(Mutex::default);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Tree(BTreeMap<PathBuf, OsString>);

impl Tree {
    pub fn built(root: &Path, build: impl FnOnce()) {
        let cell = Arc::clone(BUILT.lock().expect("cache lock").entry(Tree::captured(root)).or_default());
        cell.get_or_init(|| {
            build();
            Tree::captured(root)
        })
        .restore(root);
    }

    fn captured(root: &Path) -> Tree {
        let named = |path: PathBuf| {
            let name = path.strip_prefix(root).expect("under root").to_path_buf();
            (name, OsString::from_vec(fs::read(&path).expect("read file")))
        };
        Tree(Self::files(root).into_iter().map(named).collect())
    }

    fn files(directory: &Path) -> Vec<PathBuf> {
        let listed = fs::read_dir(directory).expect("read directory").map(|found| found.expect("entry").path());
        listed.flat_map(|path| if path.is_dir() { Self::files(&path) } else { vec![path] }).collect()
    }

    fn restore(&self, root: &Path) {
        let missing = self.0.iter().filter(|(name, _)| !root.join(name).exists());
        missing.for_each(|(name, contents)| {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap_or(root)).expect("restore directory");
            fs::write(path, contents.as_bytes()).expect("restore file");
        });
    }
}
