use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io::{self, ErrorKind};
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use crate::request::local::local_clone::vetting::cause::Cause;

#[derive(Debug)]
pub struct Procfs {
    own: u32,
}

impl Procfs {
    const ROOT: &'static str = "/proc";
    const SELF: &'static str = "/proc/self";
    const LINKS: [&'static str; 3] = ["cwd", "root", "exe"];
    const FDS: &'static str = "fd";

    pub fn held() -> Result<Vec<PathBuf>, Cause> {
        let own = Self { own: fs::metadata(Self::SELF).map_err(Cause::new)?.uid() };
        let entries = fs::read_dir(Self::ROOT).and_then(Iterator::collect::<io::Result<Vec<DirEntry>>>);
        let processes = entries.map_err(Cause::new)?.iter().map(DirEntry::path).filter(|path| Self::numbered(path));
        let held = processes.map(|process| own.scanned(&process)).collect::<Result<Vec<_>, Cause>>()?;
        Ok(held.concat())
    }

    fn numbered(path: &Path) -> bool {
        path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.bytes().all(|byte| byte.is_ascii_digit()))
    }

    fn scanned(&self, process: &Path) -> Result<Vec<PathBuf>, Cause> {
        Self::opened(process).or_else(|failure| self.excused(process, failure).map(|()| Vec::new()))
    }

    fn excused(&self, process: &Path, failure: io::Error) -> Result<(), Cause> {
        let owner = fs::metadata(process).map(|found| found.uid());
        match (failure.kind(), owner) {
            (ErrorKind::NotFound, _) => Ok(()),
            (_, Err(gone)) if gone.kind() == ErrorKind::NotFound => Ok(()),
            (_, Ok(uid)) if uid != self.own => Ok(()),
            _ => Err(Cause::new(failure)),
        }
    }

    fn opened(process: &Path) -> io::Result<Vec<PathBuf>> {
        let fds = fs::read_dir(process.join(Self::FDS))?.map(|fd| fd.map(|fd| fd.path()));
        let links = Self::LINKS.iter().map(|link| Ok(process.join(link))).chain(fds);
        let named = links.map(|link| link.and_then(fs::read_link));
        named.filter(|named| named.as_ref().err().is_none_or(|failure| failure.kind() != ErrorKind::NotFound)).collect()
    }
}
