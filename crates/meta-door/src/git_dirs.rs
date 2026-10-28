mod refusal;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
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
    const GITDIR: &str = "gitdir: ";
    const QUERY: [&str; 3] = ["rev-parse", "--path-format=absolute", "--git-common-dir"];
    const UNSET: [&str; 3] = ["GIT_DIR", "GIT_COMMON_DIR", "GIT_WORK_TREE"];

    pub fn common(directory: &Path) -> io::Result<PathBuf> {
        let directory = Self::resolved(directory)?;
        let known = KNOWN.lock().unwrap_or_else(PoisonError::into_inner).get(&directory).cloned();
        known.map_or_else(
            || {
                let common =
                    Self::read(&directory).map_or_else(|| Self::answered(&directory).map_err(io::Error::other), Ok)?;
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
        let walked = std::path::absolute(path)?.components().try_fold(PathBuf::new(), Self::stepped)?;
        Ok(walked
            .ancestors()
            .find_map(|ancestor| {
                let rest = walked.strip_prefix(ancestor).ok()?;
                ancestor.canonicalize().ok().map(|real| real.components().chain(rest.components()).collect())
            })
            .unwrap_or_else(|| walked.clone()))
    }

    fn stepped(real: PathBuf, component: Component<'_>) -> io::Result<PathBuf> {
        let next = match component {
            Component::ParentDir => real.parent().map_or_else(|| real.clone(), Path::to_path_buf),
            Component::CurDir => real,
            other => real.join(other),
        };
        match (component, next.read_link()) {
            (Component::Normal(_), Ok(_)) => {
                next.canonicalize().map_err(|_| io::Error::other(Refusal::Dangling { path: next }))
            }
            _ => Ok(next),
        }
    }

    fn read(directory: &Path) -> Option<PathBuf> {
        let own = directory.join(Self::ENTRY);
        let gitfile = fs::read_to_string(&own).ok();
        let pointed = gitfile.and_then(|text| Some(directory.join(text.lines().next()?.strip_prefix(Self::GITDIR)?)));
        let bare = (directory.join(Self::HEAD).is_file() && directory.join(Self::OBJECTS).is_dir())
            .then(|| directory.to_path_buf());
        let admin = Self::resolved(pointed.or_else(|| own.is_dir().then_some(own)).or(bare)?).ok()?;
        let shared = fs::read_to_string(admin.join(AdminFile::CommonDir.name())).ok();
        let common = shared.and_then(|text| Some(admin.join(text.lines().next()?)));
        Self::resolved(common.unwrap_or(admin)).ok()
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

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::process::Command;

    use super::GitDirs;
    use crate::Door;

    const INIT: [&str; 2] = ["init", "-q"];
    const NESTED: &str = "nested";
    const OBJECTS: &str = "objects";

    #[test]
    fn git_names_the_common_dir_of_a_subdirectory_and_an_objects_dir_alone_is_no_repository() {
        let repository = tempfile::tempdir().unwrap();
        assert!(Command::new("git").current_dir(repository.path()).args(INIT).status().unwrap().success());
        std::fs::create_dir(repository.path().join(NESTED)).unwrap();
        let mimic = tempfile::tempdir().unwrap();
        std::fs::create_dir(mimic.path().join(OBJECTS)).unwrap();
        let common = GitDirs::common(repository.path()).unwrap();
        let unknown = GitDirs::common(mimic.path()).is_err();
        let refused = Door::inspected(mimic.path()).map_err(|failure| failure.kind());
        assert_eq!(GitDirs::common(&repository.path().join(NESTED)).unwrap(), common);
        assert!(unknown && refused.err() == Some(ErrorKind::NotFound));
    }
}
