use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pair {
    Rename,
    Copy,
    Symlink,
    HardLink,
}

impl Pair {
    pub(crate) fn applied(self, from: &Path, to: &Path) -> io::Result<()> {
        match self {
            Pair::Rename => fs::rename(from, to),
            Pair::Copy => fs::copy(from, to).map(drop),
            Pair::Symlink => symlink(from, to),
            Pair::HardLink => fs::hard_link(from, to),
        }
    }
}
