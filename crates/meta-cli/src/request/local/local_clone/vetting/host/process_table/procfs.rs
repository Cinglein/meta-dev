#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io::{self, ErrorKind};
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use super::held::Held;
use super::scan::Scan;
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

    pub fn held() -> Result<Held, Cause> {
        let own = Self { own: fs::metadata(Self::SELF).map_err(Cause::new)?.uid() };
        let entries = fs::read_dir(Self::ROOT).and_then(Iterator::collect::<io::Result<Vec<DirEntry>>>);
        let processes = entries.map_err(Cause::new)?.into_iter().map(|entry| entry.path());
        let processes = processes.filter(|path| Self::numbered(path));
        processes.map(|process| own.scanned(&process)).collect()
    }

    fn numbered(path: &Path) -> bool {
        path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.bytes().all(|byte| byte.is_ascii_digit()))
    }

    fn scanned(&self, process: &Path) -> Result<Scan, Cause> {
        Self::opened(process).map(Scan::Opened).or_else(|failure| self.excused(process, failure))
    }

    fn excused(&self, process: &Path, failure: io::Error) -> Result<Scan, Cause> {
        let owner = fs::metadata(process).map(|found| found.uid());
        match (failure.kind(), owner) {
            (_, Ok(uid)) if uid != self.own => Ok(Scan::Skipped),
            (ErrorKind::NotFound, _) => Ok(Scan::Skipped),
            (_, Err(gone)) if gone.kind() == ErrorKind::NotFound => Ok(Scan::Skipped),
            (ErrorKind::PermissionDenied, _) => Ok(Scan::Uninspected),
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
