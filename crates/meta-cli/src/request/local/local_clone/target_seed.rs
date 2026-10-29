use std::fs::{self, DirEntry, File};
use std::io;
use std::path::Path;

use super::cargo_tag::CargoTag;
use crate::request::cli_failure::{CliFailure, Setup};

#[derive(Clone, Copy, Debug)]
pub struct TargetSeed<'seed> {
    clone: &'seed Path,
    worktree: &'seed Path,
}

impl<'seed> TargetSeed<'seed> {
    const LOCK: &'static str = ".cargo-lock";

    pub fn new(clone: &'seed Path, worktree: &'seed Path) -> Self {
        Self { clone, worktree }
    }

    pub fn sown(self) -> Result<(), CliFailure> {
        let targets = fs::read_dir(self.clone).map_err(CliFailure::raised::<Setup>)?;
        targets
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()) && CargoTag::tags(&entry.path()))
            .try_for_each(|target| Self::target(&target.path(), &self.worktree.join(target.file_name())))
            .map_err(CliFailure::raised::<Setup>)
    }

    fn target(from: &Path, into: &Path) -> io::Result<()> {
        fs::create_dir_all(into)?;
        fs::read_dir(from)?.flatten().try_for_each(|entry| match entry.file_type()? {
            kind if kind.is_dir() => Self::profile(&entry.path(), &into.join(entry.file_name())),
            kind if kind.is_file() => fs::copy(entry.path(), into.join(entry.file_name())).map(drop),
            _ => Ok(()),
        })
    }

    fn profile(from: &Path, into: &Path) -> io::Result<()> {
        let Ok(lock) = File::open(from.join(Self::LOCK)) else {
            return Ok(());
        };
        lock.try_lock_shared().map_or(Ok(()), |()| Self::tree(from, into))
    }

    fn tree(from: &Path, into: &Path) -> io::Result<()> {
        fs::create_dir_all(into)?;
        fs::read_dir(from)?.flatten().try_for_each(|entry| Self::copied(&entry, &into.join(entry.file_name())))
    }

    fn copied(entry: &DirEntry, into: &Path) -> io::Result<()> {
        match entry.file_type()? {
            kind if kind.is_dir() => Self::tree(&entry.path(), into),
            kind if kind.is_file() => fs::copy(entry.path(), into).map(drop),
            _ => Ok(()),
        }
    }
}
