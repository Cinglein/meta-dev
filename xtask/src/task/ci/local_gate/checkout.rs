use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use task::{CommandLine, Program, Site, Stdout};
use tempfile::TempDir;

use crate::task::ci::local_gate::{MetaRev, RecordedTimes, Unready, Worktrees};
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
    const CHANGES: [&str; 3] = ["status", "--porcelain", "--untracked-files=all"];
    const HEAD: [&str; 2] = ["rev-parse", "HEAD"];
    const SHORT_HEAD: [&str; 3] = ["rev-parse", "--short", "HEAD"];
    const GIT: &str = ".git";
    const CARGO_LOCK: &str = "Cargo.lock";
    const HELD: &str = "lock";
    const TIMES: &str = "times";
    const META_SOURCE: &str = "xtask/src/task/ci/local_gate/meta_rev.rs";

    pub fn add(repository: &Path, commit: Commit, parent: &Path) -> Result<Checkout, Failure> {
        let io = |error| Failure::from(task::Failure::Io { site: Site::File(parent.into()), error });
        let place = TempDir::new_in(parent).map_err(io)?.keep();
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
        fs::create_dir_all(parent).map_err(io)?;
        let parent = parent.canonicalize().map_err(io)?;
        let listed: Worktrees = Self::git(repository).arguments(Self::LIST).captured::<Stdout>()?.parse()?;
        listed.below(&parent).iter().filter(|place| Self::claimed(place).is_ok()).try_for_each(|place| {
            Self::git(repository).arguments(Self::REMOVE).argument(place.display()).run().inspect(|()| {
                Self::discarded(place);
            })
        })?;
        Self::git(repository).arguments(Self::PRUNE).run()?;
        Ok(parent)
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
        let git = |arguments: &[&str]| Self::git(&self.place).arguments(arguments).captured::<Stdout>();
        let head: Commit = git(&Self::HEAD)?.parse()?;
        (head == self.commit && git(&Self::CHANGES)?.is_empty()).then_some(()).ok_or(Unready::Tampered.into())
    }

    pub fn version(&self) -> Result<MetaRev, Failure> {
        fs::read_to_string(self.root().join(Self::CARGO_LOCK)).ok().and_then(|lock| lock.parse().ok()).map_or_else(
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
        fs::remove_dir_all(place).ok();
        fs::remove_file(place.with_extension(Self::HELD)).ok();
        fs::remove_file(place.with_extension(Self::TIMES)).ok();
    }

    fn claimed(place: &Path) -> io::Result<File> {
        let lock = File::create(place.with_extension(Self::HELD))?;
        lock.try_lock()?;
        Ok(lock)
    }

    fn git(directory: &Path) -> CommandLine {
        CommandLine::new(Program::Git).in_directory(directory.to_path_buf())
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
    use std::fs;
    use std::mem;
    use std::path::{Path, PathBuf};

    use super::Checkout;
    use crate::task::ci::local_gate::MetaRev;
    use crate::task::fixture::Fixture;
    use crate::task::revision::Commit;

    const IGNORE: &str = ".gitignore";
    const IGNORED: &str = "consumer/hello.txt\n";
    const EXCLUDED: &str = ".git/info/exclude";
    const CARGO_CONFIG: &str = ".cargo/config.toml";
    const PROBE: &str = "[env]\nGATE_PROBE = \"leaked\"\n";
    const COMMITTED: &str = "consumer/committed.txt";
    const HELLO: &str = "consumer/hello.txt";
    const SCOPE: &str = "consumer";
    const HEAD: &str = "HEAD";
    const HELD: &str = "lock";
    const UNKNOWN: &str = "0000000";

    impl Fixture {
        fn checked_out(&self) -> Checkout {
            self.write(IGNORE, IGNORED).write(COMMITTED, "").write(HELLO, "").write(EXCLUDED, CARGO_CONFIG);
            self.write(CARGO_CONFIG, PROBE).branch();
            self.added()
        }

        fn added(&self) -> Checkout {
            let head: Commit = self.revision(HEAD).parse().unwrap();
            let parent = Checkout::sweep(&self.path(SCOPE), &self.beside()).unwrap();
            Checkout::add(&self.path(SCOPE), head, &parent).unwrap()
        }
    }

    impl Checkout {
        fn abandoned(self) -> PathBuf {
            self.lock.unlock().unwrap();
            let place = self.place.clone();
            mem::forget(self);
            place
        }
    }

    #[test]
    fn a_checkout_outside_the_repository_holds_exactly_the_commit_scoped_like_the_workspace() {
        let fixture = Fixture::new();
        let checkout = fixture.checked_out();
        assert!(checkout.root().join("committed.txt").exists() && !checkout.root().join("hello.txt").exists());
        assert!(checkout.root().ancestors().all(|directory| !directory.join(CARGO_CONFIG).exists()));
        let local = checkout.version().is_ok_and(|version| version == MetaRev::local());
        assert!(checkout.scope() == Path::new(SCOPE) && checkout.intact().is_ok() && local);
    }

    #[test]
    fn a_dropped_or_failed_checkout_leaves_nothing_behind() {
        let fixture = Fixture::new();
        let checkout = fixture.checked_out();
        let parent = checkout.place.parent().unwrap().to_path_buf();
        drop(checkout);
        let unknown: Commit = UNKNOWN.parse().unwrap();
        assert!(Checkout::add(&fixture.path(SCOPE), unknown, &parent).is_err());
        assert!(fs::read_dir(&parent).unwrap().next().is_none());
    }

    #[test]
    fn a_sweep_removes_only_checkouts_no_running_gate_holds() {
        let fixture = Fixture::new();
        let stale = fixture.checked_out().abandoned();
        let live = fixture.added();
        Checkout::sweep(&fixture.path(SCOPE), &fixture.beside()).unwrap();
        assert!(!stale.exists() && !stale.with_extension(HELD).exists() && live.root().exists());
    }
}
