mod refusal;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, PoisonError};

use crate::admin_file::AdminFile;
use refusal::Refusal;

#[derive(Debug)]
pub struct GitDirs;

static KNOWN: Mutex<BTreeMap<PathBuf, PathBuf>> = Mutex::new(BTreeMap::new());

impl GitDirs {
    const ENTRY: &str = ".git";
    const HEAD: &str = "HEAD";
    const OBJECTS: &str = "objects";
    const QUERY: [&str; 3] = ["rev-parse", "--path-format=absolute", "--git-common-dir"];
    const UNSET: [&str; 3] = ["GIT_DIR", "GIT_COMMON_DIR", "GIT_WORK_TREE"];

    pub fn common(directory: &Path) -> io::Result<PathBuf> {
        let directory = Self::resolved(directory)?;
        let known = KNOWN.lock().unwrap_or_else(PoisonError::into_inner).get(&directory).cloned();
        known.map_or_else(
            || {
                let common =
                    Self::plain(&directory).map_or_else(|| Self::answered(&directory).map_err(io::Error::other), Ok)?;
                KNOWN.lock().unwrap_or_else(PoisonError::into_inner).insert(directory.clone(), common.clone());
                Ok(common)
            },
            Ok,
        )
    }

    pub fn outside(path: PathBuf) -> io::Result<PathBuf> {
        let real = Self::resolved(&path)?;
        let repository = real.ancestors().find(|directory| {
            directory.join(Self::ENTRY).exists()
                || (directory.join(Self::HEAD).is_file() && directory.join(Self::OBJECTS).is_dir())
        });
        let inside = repository.map(Self::common).transpose()?.filter(|common| real.starts_with(common));
        inside.map_or(Ok(path), |common| Err(io::Error::other(Refusal::Inside { path: real.clone(), git: common })))
    }

    pub fn within(path: PathBuf, common: &Path, area: &OsStr) -> io::Result<PathBuf> {
        let real = Self::resolved(&path)?;
        let area = Self::resolved(common)?.join(area);
        if real.starts_with(&area) { Ok(path) } else { Err(io::Error::other(Refusal::Escaped { path: real, area })) }
    }

    pub fn resolved(path: impl AsRef<Path>) -> io::Result<PathBuf> {
        std::path::absolute(path)?.components().try_fold(PathBuf::new(), Self::stepped)
    }

    fn stepped(real: PathBuf, component: Component<'_>) -> io::Result<PathBuf> {
        let next = match component {
            Component::ParentDir => real.parent().map_or_else(|| real.clone(), Path::to_path_buf),
            Component::CurDir => real,
            other => real.join(other),
        };
        match (component, next.symlink_metadata()) {
            (Component::Normal(_), Ok(meta)) if meta.is_symlink() => {
                next.canonicalize().map_err(|_| io::Error::other(Refusal::Dangling { path: next }))
            }
            _ => Ok(next),
        }
    }

    fn plain(directory: &Path) -> Option<PathBuf> {
        let own = directory.join(Self::ENTRY);
        (own.is_dir() && !own.join(AdminFile::CommonDir.name()).exists()).then_some(own)
    }

    fn answered(directory: &Path) -> Result<PathBuf, Refusal> {
        let command = Self::UNSET.iter().fold(Command::new("git"), |mut command, name| {
            command.env_remove(name);
            command
        });
        let output = { command }.arg("-C").arg(directory).args(Self::QUERY).output();
        let output = output.map_err(|failure| Refusal::Unrun { directory: directory.to_path_buf(), failure })?;
        let text = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<PathBuf> = text.lines().filter_map(|line| Self::resolved(Path::new(line)).ok()).collect();
        match (output.status.success(), lines.as_slice()) {
            (true, [common]) => Ok(common.clone()),
            _ => Err(Refusal::Unknown {
                directory: directory.to_path_buf(),
                said: String::from_utf8_lossy(&output.stderr).into_owned(),
            }),
        }
    }
}
