mod refusal;

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
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
        let directory = Self::real(directory);
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
        let real = Self::real(&std::path::absolute(&path)?);
        let repository = real.ancestors().find(|directory| {
            directory.join(Self::ENTRY).exists()
                || (directory.join(Self::HEAD).is_file() && directory.join(Self::OBJECTS).is_dir())
        });
        let inside = repository.map(Self::common).transpose()?.filter(|common| real.starts_with(common));
        inside.map_or(Ok(path), |common| Err(io::Error::other(Refusal::Inside { path: real.clone(), git: common })))
    }

    fn real(path: &Path) -> PathBuf {
        path.ancestors()
            .find_map(|ancestor| {
                let rest = path.strip_prefix(ancestor).ok()?;
                ancestor.canonicalize().ok().map(|real| real.components().chain(rest.components()).collect())
            })
            .unwrap_or_else(|| path.to_path_buf())
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
        let lines: Vec<PathBuf> = text.lines().map(|line| Self::real(Path::new(line))).collect();
        match (output.status.success(), lines.as_slice()) {
            (true, [common]) => Ok(common.clone()),
            _ => Err(Refusal::Unknown {
                directory: directory.to_path_buf(),
                said: String::from_utf8_lossy(&output.stderr).into_owned(),
            }),
        }
    }
}
