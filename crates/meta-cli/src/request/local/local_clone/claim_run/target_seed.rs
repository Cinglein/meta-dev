use std::fs::{self, DirEntry, File};
use std::io;
use std::path::Path;

use crate::request::local::local_clone::cargo_tag::CargoTag;

#[derive(Clone, Copy, Debug)]
pub struct TargetSeed<'seed> {
    clone: &'seed Path,
    worktree: &'seed Path,
}

impl<'seed> TargetSeed<'seed> {
    const LOCK: &'static str = ".cargo-lock";
    const COPY_ON_WRITE: bool = cfg!(target_os = "macos");

    pub fn new(clone: &'seed Path, worktree: &'seed Path) -> Self {
        Self { clone, worktree }
    }

    pub fn sown(self) {
        if !Self::COPY_ON_WRITE {
            eprintln!("meta: warning: target seed skipped, copy-on-write seeding is macOS only");
        } else if let Err(cause) = self.attempted() {
            eprintln!("meta: warning: target seed failed, cargo will rebuild: {cause}");
        }
    }

    fn attempted(self) -> io::Result<()> {
        let targets = fs::read_dir(self.clone)?;
        targets
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()) && CargoTag::tags(&entry.path()))
            .try_for_each(|target| self.target(&target))
    }

    fn target(self, target: &DirEntry) -> io::Result<()> {
        let into = self.worktree.join(target.file_name());
        if into.exists() {
            return Ok(());
        }
        fs::create_dir_all(&into)?;
        Self::profiles(target, &into).inspect_err(|_| {
            drop(fs::remove_dir_all(&into));
        })
    }

    fn profiles(target: &DirEntry, into: &Path) -> io::Result<()> {
        fs::read_dir(target.path())?.flatten().try_for_each(|entry| {
            let held = entry.file_type()?.is_dir().then(|| Self::locked(&entry.path()));
            match held {
                Some(None) => Ok(()),
                Some(Some(_)) | None => Self::copied(&entry, &into.join(entry.file_name())),
            }
        })
    }

    fn locked(profile: &Path) -> Option<File> {
        File::open(profile.join(Self::LOCK)).ok().filter(|lock| lock.try_lock_shared().is_ok())
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
