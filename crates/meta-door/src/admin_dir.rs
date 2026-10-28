use std::ffi::OsStr;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use crate::admin_file::AdminFile;
use crate::git_dirs::GitDirs;
use crate::kind::Kind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminDir(PathBuf);

impl AdminDir {
    const REFUSED: [&str; 4] = ["refs", "packed-refs", "reftable", "logs"];
    const OBJECTS: &str = "objects";

    pub(crate) fn new(real: PathBuf) -> AdminDir {
        AdminDir(real)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn text(&self, file: AdminFile) -> io::Result<String> {
        fs::read_to_string(self.0.join(file.name()))
    }

    #[must_use]
    pub fn kind(&self, file: AdminFile) -> Kind {
        Kind::found(self.0.join(file.name()).symlink_metadata())
    }

    pub fn listed(&self) -> io::Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.0)?.map(|entry| entry.map(|entry| entry.path()));
        let alternates = self.0.join(AdminFile::Alternates.name());
        let lent = (self.kind(AdminFile::Alternates) == Kind::File).then_some(alternates);
        let admitted = entries.collect::<io::Result<Vec<PathBuf>>>()?.into_iter().chain(lent);
        Ok(admitted.filter(|path| self.admitted(path).is_ok()).collect())
    }

    pub fn described(&self, path: &Path) -> io::Result<Metadata> {
        self.admitted(path)?.symlink_metadata()
    }

    pub fn object_written(&self, loose: &Path) -> io::Result<SystemTime> {
        (loose.parent().and_then(Path::parent) == Some(Path::new(Self::OBJECTS))
            && loose.components().all(|part| matches!(part, Component::Normal(_))))
        .then_some(())
        .ok_or(io::ErrorKind::PermissionDenied)?;
        self.0.join(loose).symlink_metadata()?.modified()
    }

    fn admitted(&self, path: &Path) -> io::Result<PathBuf> {
        let rest = path.strip_prefix(&self.0).map_err(|_| io::Error::from(io::ErrorKind::PermissionDenied))?;
        (Self::top(rest) || rest == Path::new(AdminFile::Alternates.name()))
            .then_some(())
            .ok_or(io::ErrorKind::PermissionDenied)?;
        GitDirs::within(path.to_path_buf(), &self.0, OsStr::new(""))
    }

    fn top(rest: &Path) -> bool {
        let mut parts = rest.components();
        let first = parts.next();
        parts.next().is_none()
            && matches!(first, Some(Component::Normal(name))
                if !Self::REFUSED.iter().any(|refused| name.eq_ignore_ascii_case(refused)))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use crate::Door;

    const COMMITTED: [&str; 9] =
        ["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "x"];
    const PACKED: [&str; 2] = ["pack-refs", "--all"];
    const INIT: [&str; 2] = ["init", "-q"];
    const ENTRY: &str = ".git";
    const HEADS: &str = "refs/heads";
    const REFS: [&str; 4] = ["refs", "packed-refs", "logs", "reftable"];
    const ESCAPE: &str = "objects/../refs";

    #[test]
    fn an_admin_dir_never_lists_or_describes_refs_even_after_they_are_packed() {
        let repository = tempfile::tempdir().unwrap();
        let git = |words: &[&str]| Command::new("git").current_dir(repository.path()).args(words).status().unwrap();
        assert!([&INIT[..], &COMMITTED[..], &PACKED[..]].iter().all(|words| git(words).success()));
        let admin = Door::inspected(&repository.path().join(ENTRY)).unwrap();
        let listed = admin.listed().unwrap();
        let named = |path: &Path| REFS.iter().any(|name| path.file_name().is_some_and(|file| file == *name));
        let refused =
            [HEADS, ESCAPE].iter().chain(&REFS).all(|name| admin.described(&admin.path().join(name)).is_err());
        assert!(!listed.is_empty() && !listed.iter().any(|path| named(path)) && refused);
    }
}
