use meta_door::kinds::File;
use meta_door::{Change, Door, Opening, Scratch};
use std::io;
use std::path::{Path, PathBuf};

use task::{CommandLine, Program, Site, Stdout};

use crate::task::ci::local_gate::{MetaRev, RecordedTimes, Toolchain, Unready, Worktrees};
use crate::task::failure::Failure;
use crate::task::revision::Commit;

#[derive(Debug)]
pub struct Checkout {
    repository: PathBuf,
    place: PathBuf,
    scope: PathBuf,
    commit: Commit,
    lock: File,
}

impl Checkout {
    const REMOVE: [&str; 3] = ["worktree", "remove", "--force"];
    const ADD: [&str; 3] = ["worktree", "add", "--detach"];
    const PRUNE: [&str; 2] = ["worktree", "prune"];
    const LIST: [&str; 3] = ["worktree", "list", "--porcelain"];
    const CHANGES: [&str; 6] = ["status", "--porcelain", "--ignored", "--untracked-files=all", "--", ":/"];
    const UNTRACKED_LOCK: &str = ":(exclude)Cargo.lock";
    const TRACKED: [&str; 4] = ["ls-files", "--error-unmatch", "--", "Cargo.lock"];
    const NO_HOOKS: [&str; 2] = ["-c", "core.hooksPath=/dev/null"];
    const RUSTUP: &str = "rustup";
    const ACTIVE_TOOLCHAIN: [&str; 2] = ["show", "active-toolchain"];
    const RUSTUP_ENVIRONMENT: [&str; 4] = ["PATH", "HOME", "RUSTUP_HOME", "CARGO_HOME"];
    const HEAD: [&str; 2] = ["rev-parse", "HEAD"];
    const SHORT_HEAD: [&str; 3] = ["rev-parse", "--short", "HEAD"];
    const GIT: &str = ".git";
    const CARGO_LOCK: &str = "Cargo.lock";
    const HELD: &str = "lock";
    const TIMES: &str = "times";
    const META_SOURCE: &str = "xtask/src/task/ci/local_gate/meta_rev.rs";

    pub fn add(repository: &Path, commit: Commit, parent: &Path) -> Result<Checkout, Failure> {
        let io = |error| Failure::from(task::Failure::Io { site: Site::File(parent.into()), error });
        let place = Scratch::made(parent).map_err(io)?.kept();
        let lock = Self::claimed(&place).map_err(io)?;
        Self::git(repository)
            .arguments(Self::ADD)
            .argument(place.display())
            .argument(&commit)
            .captured::<Stdout>()
            .inspect_err(|_| Self::discarded(&place))?;
        let scope = Self::scope_of(repository);
        Ok(Checkout { repository: repository.to_path_buf(), place, scope, commit, lock })
    }

    pub fn scope_of(repository: &Path) -> PathBuf {
        let top = repository.ancestors().find(|directory| directory.join(Self::GIT).exists()).unwrap_or(repository);
        repository.strip_prefix(top).unwrap_or(Path::new("")).to_path_buf()
    }

    pub fn sweep(repository: &Path, parent: &Path) -> Result<PathBuf, Failure> {
        let io = |error| task::Failure::Io { site: Site::File(parent.into()), error };
        Door::changed(parent, Change::DirAll).map_err(io)?;
        let parent = parent.canonicalize().map_err(io)?;
        let listed: Worktrees = Self::git(repository).arguments(Self::LIST).captured::<Stdout>()?.parse()?;
        listed.below(&parent).iter().filter(|place| Self::claimed(place).is_ok()).try_for_each(|place| {
            Self::git(repository).arguments(Self::REMOVE).argument(place.display()).run().inspect(|()| {
                Self::discarded(place);
            })
        })?;
        Self::git(repository).arguments(Self::PRUNE).run()?;
        Self::orphans(&parent).iter().filter(|place| Self::claimed(place).is_ok()).for_each(|place| {
            Self::discarded(place);
        });
        Ok(parent)
    }

    pub fn holds(&self, path: &Path) -> bool {
        path.canonicalize().is_ok_and(|real| real.starts_with(&self.place))
    }

    pub fn above(&self) -> Option<&Path> {
        self.place.parent()
    }

    pub fn root(&self) -> PathBuf {
        self.place.join(&self.scope)
    }

    pub fn times(&self) -> RecordedTimes {
        RecordedTimes::new(self.place.with_extension(Self::TIMES))
    }

    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    pub fn scope(&self) -> &Path {
        &self.scope
    }

    pub fn intact(&self) -> Result<(), Failure> {
        let head: Commit = Self::git(&self.place).arguments(Self::HEAD).captured::<Stdout>()?.parse()?;
        let untracked_lock = Self::git(&self.root())
            .arguments(Self::TRACKED)
            .captured::<Stdout>()
            .is_err()
            .then_some(Self::UNTRACKED_LOCK);
        let changes =
            Self::git(&self.root()).arguments(Self::CHANGES).arguments(untracked_lock).captured::<Stdout>()?;
        (head == self.commit && changes.is_empty()).then_some(()).ok_or(Unready::Tampered.into())
    }

    pub fn toolchain(&self) -> Result<Toolchain, Failure> {
        CommandLine::new(Program::Executable(PathBuf::from(Self::RUSTUP)))
            .in_directory(self.root())
            .arguments(Self::ACTIVE_TOOLCHAIN)
            .inheriting_only(Self::RUSTUP_ENVIRONMENT)
            .captured::<Stdout>()?
            .parse()
    }

    pub fn version(&self) -> Result<MetaRev, Failure> {
        Door::read(self.root().join(Self::CARGO_LOCK)).ok().and_then(|lock| lock.parse().ok()).map_or_else(
            || {
                if self.place.join(Self::META_SOURCE).exists() {
                    Self::git(&self.place).arguments(Self::SHORT_HEAD).captured::<Stdout>()?.parse()
                } else {
                    Ok(MetaRev::local())
                }
            },
            Ok,
        )
    }

    fn discarded(place: &Path) {
        Door::changed(place, Change::RemoveDirAll).ok();
        Door::changed(place.with_extension(Self::HELD), Change::RemoveFile).ok();
        Door::changed(place.with_extension(Self::TIMES), Change::RemoveFile).ok();
    }

    fn claimed(place: &Path) -> io::Result<File> {
        let lock = Door::opened(place.with_extension(Self::HELD), Opening::Truncate)?;
        lock.try_lock()?;
        Ok(lock)
    }

    fn git(directory: &Path) -> CommandLine {
        CommandLine::new(Program::Git).in_directory(directory.to_path_buf()).arguments(Self::NO_HOOKS)
    }

    fn orphans(parent: &Path) -> Vec<PathBuf> {
        Door::listed(parent)
            .map(|entries| {
                entries
                    .into_iter()
                    .filter(|path| path.is_file())
                    .map(|path| path.with_extension(""))
                    .filter(|place| !place.exists())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Drop for Checkout {
    fn drop(&mut self) {
        Self::git(&self.repository)
            .arguments(Self::REMOVE)
            .argument(self.place.display())
            .run()
            .unwrap_or_else(|failure| eprintln!("{failure}"));
        self.lock.unlock().ok();
        Self::discarded(&self.place);
    }
}

#[cfg(test)]
mod tests {
    use meta_door::{Door, Pair};
    use std::mem;
    use std::path::{Path, PathBuf};

    use super::Checkout;
    use crate::task::ci::local_gate::MetaRev;
    use crate::task::fixture::Fixture;
    use crate::task::revision::Commit;

    const IGNORE: &str = ".gitignore";
    const IGNORED: &str = "consumer/hello.txt\n.cargo/config.toml\n";
    const CARGO_CONFIG: &str = ".cargo/config.toml";
    const PROBE: &str = "[env]\nGATE_PROBE = \"leaked\"\n";
    const COMMITTED: &str = "consumer/committed.txt";
    const HELLO: &str = "consumer/hello.txt";
    const SCOPE: &str = "consumer";
    const HEAD: &str = "HEAD";
    const HELD: &str = "lock";
    const UNKNOWN: &str = "0000000";
    const ORPHAN: &str = ".tmpgone.times";
    const TRACKED_LOCK: &str = "consumer/Cargo.lock";
    const LOCKFILE: &str = "Cargo.lock";
    const LOCKED: &str = "version = 4\n";
    const LOCKED_AGAIN: &str = "version = 3\n";
    const LINK: &str = "outside";

    impl Fixture {
        fn checked_out(&self) -> Checkout {
            self.write(IGNORE, IGNORED).write(COMMITTED, "").write(HELLO, "");
            self.write(TRACKED_LOCK, LOCKED);
            self.write(CARGO_CONFIG, PROBE).branch();
            self.added()
        }

        fn added(&self) -> Checkout {
            let head: Commit = self.revision(HEAD).parse().unwrap();
            let parent = Checkout::sweep(&self.path(SCOPE), &self.beside()).unwrap();
            Checkout::add(&self.path(SCOPE), head, &parent).unwrap()
        }
    }

    impl Fixture {
        fn abandoned(checkout: Checkout) -> PathBuf {
            checkout.lock.unlock().unwrap();
            let place = checkout.place.clone();
            mem::forget(checkout);
            place
        }
    }

    #[test]
    fn a_checkout_outside_the_repository_holds_exactly_the_commit_scoped_like_the_workspace() {
        let fixture = Fixture::new();
        let checkout = fixture.checked_out();
        assert!(checkout.root().join("committed.txt").exists() && !checkout.root().join("hello.txt").exists());
        let local = checkout.version().is_ok_and(|version| version == MetaRev::local());
        assert!(checkout.scope() == Path::new(SCOPE) && checkout.intact().is_ok() && local);
        Door::write(checkout.root().join(LOCKFILE), LOCKED_AGAIN).unwrap();
        let relocked = checkout.intact().is_err();
        Door::paired(fixture.root(), checkout.root().join(LINK), Pair::Symlink).unwrap();
        let outside = checkout.above().is_some_and(|above| checkout.root().starts_with(above))
            && checkout.holds(&checkout.root())
            && !checkout.holds(&checkout.root().join(LINK).join(COMMITTED));
        let excluded = checkout.root().with_file_name(CARGO_CONFIG);
        assert!(relocked && outside && !checkout.root().starts_with(fixture.root()) && !excluded.exists());
    }

    #[test]
    fn a_dropped_or_failed_checkout_leaves_nothing_behind() {
        let fixture = Fixture::new();
        let checkout = fixture.checked_out();
        let parent = checkout.place.parent().unwrap().to_path_buf();
        drop(checkout);
        let unknown: Commit = UNKNOWN.parse().unwrap();
        assert!(Checkout::add(&fixture.path(SCOPE), unknown, &parent).is_err());
        assert_eq!(Door::listed(&parent).unwrap(), Vec::<PathBuf>::new());
    }

    #[test]
    fn a_sweep_removes_only_checkouts_no_running_gate_holds() {
        let fixture = Fixture::new();
        let stale = Fixture::abandoned(fixture.checked_out());
        let live = fixture.added();
        let orphan = fixture.beside().join(ORPHAN);
        Door::write(&orphan, "").unwrap();
        Checkout::sweep(&fixture.path(SCOPE), &fixture.beside()).unwrap();
        assert!(!stale.exists() && !stale.with_extension(HELD).exists() && live.root().exists() && !orphan.exists());
    }
}
