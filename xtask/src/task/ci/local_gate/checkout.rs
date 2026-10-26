use std::path::{Path, PathBuf};

use task::{CommandLine, Program, Stdout, Workspace};

use crate::task::failure::Failure;
use crate::task::revision::Commit;

#[derive(Debug)]
pub struct Checkout {
    repository: PathBuf,
    place: PathBuf,
    scope: PathBuf,
    commit: Commit,
}

impl Checkout {
    const REMOVE: [&str; 3] = ["worktree", "remove", "--force"];
    const ADD: [&str; 3] = ["worktree", "add", "--detach"];
    const GIT: &str = ".git";

    pub fn add(repository: &Path, commit: Commit, place: PathBuf) -> Result<Checkout, Failure> {
        let git = || CommandLine::new(Program::Git).in_directory(repository.to_path_buf());
        git().arguments(Self::REMOVE).argument(place.display()).captured::<Stdout>().ok();
        git().arguments(Self::ADD).argument(place.display()).argument(&commit).captured::<Stdout>()?;
        let top = repository.ancestors().find(|directory| directory.join(Self::GIT).exists()).unwrap_or(repository);
        let scope = repository.strip_prefix(top).unwrap_or(Path::new("")).to_path_buf();
        Ok(Checkout { repository: repository.to_path_buf(), place, scope, commit })
    }

    pub fn workspace(&self, original: &Workspace) -> Result<Workspace, Failure> {
        Ok(original.checked_out(&self.root())?)
    }

    pub fn root(&self) -> PathBuf {
        self.place.join(&self.scope)
    }

    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    pub fn scope(&self) -> &Path {
        &self.scope
    }
}

impl Drop for Checkout {
    fn drop(&mut self) {
        CommandLine::new(Program::Git)
            .in_directory(self.repository.clone())
            .arguments(Self::ADD)
            .argument(self.place.display())
            .run()
            .unwrap_or_else(|failure| eprintln!("{failure}"));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Checkout;
    use crate::task::fixture::Fixture;
    use crate::task::revision::Commit;

    const IGNORE: &str = ".gitignore";
    const IGNORED: &str = "target/\nconsumer/hello.txt\n";
    const MANIFEST: &str = "consumer/Cargo.toml";
    const HELLO: &str = "consumer/hello.txt";
    const PLACE: &str = "target/local-gate";
    const SCOPE: &str = "consumer";
    const HEAD: &str = "HEAD";

    impl Fixture {
        fn checked_out(&self) -> Checkout {
            self.write(IGNORE, IGNORED).write(MANIFEST, "").write(HELLO, "").branch();
            let head: Commit = self.revision(HEAD).parse().unwrap();
            Checkout::add(&self.path(SCOPE), head, self.path(PLACE)).unwrap()
        }
    }

    #[test]
    fn a_checkout_holds_exactly_the_commit_scoped_like_the_workspace() {
        let fixture = Fixture::new();
        let checkout = fixture.checked_out();
        assert!(checkout.root().join("Cargo.toml").exists() && !checkout.root().join("hello.txt").exists());
        assert_eq!(checkout.scope(), Path::new(SCOPE));
    }

    #[test]
    fn a_dropped_checkout_is_removed() {
        let fixture = Fixture::new();
        drop(fixture.checked_out());
        assert!(!fixture.path(PLACE).exists());
    }
}
