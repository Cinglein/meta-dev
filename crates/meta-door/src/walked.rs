use std::ffi::OsStr;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use crate::kind::Kind;

#[derive(Debug)]
pub struct Walked {
    relative: PathBuf,
    kind: Kind,
    described: Option<Metadata>,
}

impl Walked {
    const ENTRY: &str = ".git";
    const HEAD: &str = "HEAD";
    const OBJECTS: &str = "objects";

    pub(crate) fn under(root: &Path, relative: &Path) -> Vec<Walked> {
        let named = |entry: fs::DirEntry| relative.join(entry.file_name());
        let listed = fs::read_dir(root.join(relative))
            .and_then(|entries| entries.map(|entry| entry.map(named)).collect::<io::Result<Vec<PathBuf>>>());
        listed.map_or_else(
            |_| vec![Self::unreadable(relative)],
            |paths| paths.into_iter().flat_map(|path| Self::entered(root, path)).collect(),
        )
    }

    #[must_use]
    pub fn relative(&self) -> &Path {
        &self.relative
    }

    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    #[must_use]
    pub fn described(&self) -> Option<&Metadata> {
        self.described.as_ref()
    }

    fn entered(root: &Path, relative: PathBuf) -> Vec<Walked> {
        let path = root.join(&relative);
        let described = path.symlink_metadata().ok();
        let classified = described.as_ref().map_or(Kind::Unreadable, Kind::classified);
        let kind = Some(classified).filter(|kind| *kind != Kind::Dir || !Self::admin(&path)).unwrap_or(Kind::Admin);
        let below = match kind {
            Kind::Dir => Self::under(root, &relative),
            _ => Vec::new(),
        };
        std::iter::once(Walked { relative, kind, described }).chain(below).collect()
    }

    fn admin(path: &Path) -> bool {
        path.file_name().is_some_and(|name| name.eq_ignore_ascii_case(OsStr::new(Self::ENTRY)))
            || (path.join(Self::HEAD).is_file() && path.join(Self::OBJECTS).is_dir())
    }

    fn unreadable(relative: &Path) -> Walked {
        Walked { relative: relative.to_path_buf(), kind: Kind::Unreadable, described: None }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Door;
    use crate::kind::Kind;

    const NESTED: &str = "nested/.git/objects";
    const BARE_OBJECTS: &str = "bare/objects";
    const BARE_HEAD: &str = "bare/HEAD";
    const PLAIN: &str = "plain/objects";
    const ADMIN: [&str; 2] = ["bare", "nested/.git"];

    #[test]
    fn a_walk_stops_at_git_dirs_and_enters_everything_else() {
        let root = tempfile::tempdir().unwrap();
        assert!([NESTED, BARE_OBJECTS, PLAIN].iter().all(|dir| std::fs::create_dir_all(root.path().join(dir)).is_ok()));
        std::fs::write(root.path().join(BARE_HEAD), "").unwrap();
        let walked = Door::walked(root.path()).unwrap();
        let mut admin: Vec<&Path> =
            walked.iter().filter(|walked| walked.kind() == Kind::Admin).map(super::Walked::relative).collect();
        admin.sort();
        let entered = walked.iter().any(|walked| walked.relative() == Path::new(PLAIN));
        let inside = walked.iter().any(|walked| walked.relative().starts_with(NESTED));
        assert!(admin == ADMIN.map(Path::new) && entered && !inside);
    }
}
