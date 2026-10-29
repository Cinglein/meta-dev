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
        match Self::COPY_ON_WRITE.then(|| self.attempted()) {
            None => eprintln!("meta: warning: target seed skipped, copy-on-write seeding is macOS only"),
            Some(Ok(())) => {}
            Some(Err(cause)) => eprintln!("meta: warning: target seed failed, cargo will rebuild: {cause}"),
        }
    }

    fn attempted(self) -> io::Result<()> {
        fs::read_dir(self.clone)?
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()) && CargoTag::tags(&entry.path()))
            .try_for_each(|target| self.target(&target))
    }

    fn target(self, target: &DirEntry) -> io::Result<()> {
        let into = self.worktree.join(target.file_name());
        if into.exists() {
            Ok(())
        } else {
            fs::create_dir_all(&into)
                .and_then(|()| Self::profiles(&target.path(), &into))
                .inspect_err(|_| drop(fs::remove_dir_all(&into)))
        }
    }

    fn profiles(from: &Path, into: &Path) -> io::Result<()> {
        fs::read_dir(from)?.flatten().try_for_each(|entry| {
            let held = File::open(entry.path().join(Self::LOCK)).ok().filter(|lock| lock.try_lock_shared().is_ok());
            if !entry.file_type()?.is_dir() || held.is_some() {
                Self::copied(&entry, &into.join(entry.file_name()))
            } else {
                Ok(())
            }
        })
    }

    fn copied(entry: &DirEntry, into: &Path) -> io::Result<()> {
        match entry.file_type()? {
            kind if kind.is_dir() => {
                fs::create_dir_all(into)?;
                fs::read_dir(entry.path())?
                    .flatten()
                    .try_for_each(|inner| Self::copied(&inner, &into.join(inner.file_name())))
            }
            kind if kind.is_file() => fs::copy(entry.path(), into).map(drop),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TargetSeed;

    const TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n# created by cargo\n";
    const FILES: [&str; 4] = ["t/CACHEDIR.TAG", "t/debug/.cargo-lock", "t/debug/deps/libx.rlib", "t/doc/page"];

    #[test]
    fn a_seed_copies_the_tag_and_locked_profiles_and_skips_unlocked_ones() {
        let clone = tempfile::tempdir().unwrap();
        let worktree = tempfile::tempdir().unwrap();
        for path in FILES.map(|file| clone.path().join(file)) {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, TAG).unwrap();
        }
        TargetSeed::new(clone.path(), worktree.path()).attempted().unwrap();
        assert_eq!(FILES.map(|path| worktree.path().join(path).exists()), [true, true, true, false]);
    }
}
