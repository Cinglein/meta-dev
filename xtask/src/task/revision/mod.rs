mod commit;
mod foreign_emails;
mod git_paths;
mod git_reads;
mod tracked_path;

use std::path::{Path, PathBuf};

use meta_git::{CommitEmails, MergeBase, TreeText, WorktreeFiles};
use task::BranchName;

use crate::task::failure::Failure;
pub use commit::Commit;
pub use foreign_emails::ForeignEmails;
pub use git_paths::GitPaths;
pub use git_reads::GitReads;
pub use tracked_path::TrackedPath;

#[derive(Debug)]
pub struct Revision {
    git: GitReads,
    commit: Commit,
}

impl Revision {
    pub fn merge_base(root: &Path, base: &BranchName) -> Result<Revision, Failure> {
        let git = GitReads::new(root);
        let found = git.read(&MergeBase::from(&*base.as_ref().to_string_lossy()));
        let commit = found.map_err(|_| task::Failure::NoMergeBase)?;
        Ok(Revision { commit: Commit::from(commit), git })
    }

    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    pub fn in_database(database: &Path, commit: Commit) -> Revision {
        Revision { git: GitReads::new(database), commit }
    }

    pub fn present(&self, path: &Path) -> Result<Option<String>, Failure> {
        self.found(self.git.below()?.join(path))
    }

    pub fn tracked(&self, path: TrackedPath) -> Result<Option<String>, Failure> {
        self.found(path.path().to_path_buf())
    }

    fn found(&self, path: PathBuf) -> Result<Option<String>, Failure> {
        self.git.read(&TreeText::new(self.commit.object_id()?, path))
    }

    pub fn changed_paths(&self) -> Result<GitPaths, Failure> {
        let listed = self.git.files(WorktreeFiles::ChangedSince(self.commit.object_id()?))?;
        Ok(listed.and(self.git.files(WorktreeFiles::Untracked)?))
    }

    pub fn noreply_off(root: &Path, base: &BranchName) -> Result<(), Failure> {
        let emails = GitReads::new(root).read(&CommitEmails::from(&*base.as_ref().to_string_lossy()))?;
        ForeignEmails::refused(&emails.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::fs;

    use meta_golden::GoldenFailure;
    use meta_run::GitArgv;
    use settings::EmbeddedDefaults;

    use super::{Commit, ForeignEmails, GitPaths, GitReads, Revision};
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;
    use crate::task::lint::Lint;
    use crate::task::prior_policy::PriorPolicy;
    use crate::task::ratchet::Baseline;
    use crate::task::surfaces::Surfaces;
    use task::{BaselineFile, Branch, Ledger, Policy};

    const PARENT_POLICY: &str = "tooling/policy.toml";
    const PARENT_BASELINE: &str = "lint-baseline.toml";
    const POLICY: &str = "consumer/tooling/policy.toml";
    const BASELINE: &str = "consumer/lint-baseline.toml";
    const PARENT_TEXT: &str = "surfaces = []\nbaseline = \"lint-baseline.toml\"\n";
    const OWN_TEXT: &str = "surfaces = [\"tooling/policy.toml\"]\nbaseline = \"lint-baseline.toml\"\n";
    const LOOSE: &str = "crate-graph = 5\n";
    const ONE: &str = "crate-graph = 1\n";
    const TWO: &str = "crate-graph = 2\n";
    const LISTED: &str = "- tooling/policy.toml\n";
    const CONSUMER: &str = "consumer";
    const LOST: &str = "0123456789abcdef0123456789abcdef01234567";
    const CHANGED: &str =
        "surfaces = [\"tooling/policy.toml\"]\nbaseline = \"lint-baseline.toml\"\nbit_boundary = []\n";
    const VIEWED: [GitArgv; 6] = [
        GitArgv::MergeBase,
        GitArgv::LsTree,
        GitArgv::ChangedNames,
        GitArgv::UntrackedNames,
        GitArgv::TrackedNames,
        GitArgv::LogEmails,
    ];

    impl Revision {
        fn view(call: GitArgv, out: &str) -> Box<dyn Debug> {
            match call {
                GitArgv::MergeBase => Box::new(out.parse::<Commit>()),
                GitArgv::LogEmails => Box::new(ForeignEmails::refused(out)),
                _ => Box::new(out.parse::<GitPaths>()),
            }
        }
    }

    #[test]
    fn every_recorded_revision_read_parses_to_its_golden_view_and_a_failed_read_fails_closed()
    -> Result<(), GoldenFailure> {
        assert!(GitReads::all_viewed(&VIEWED, Revision::view)?);
        let unread = tempfile::tempdir().unwrap();
        let lost = Revision::in_database(unread.path(), LOST.parse().unwrap());
        assert!(lost.present(std::path::Path::new(PARENT_POLICY)).is_err());
        Ok(())
    }

    #[test]
    fn a_workspace_below_the_git_top_level_reads_its_own_prior_policy_baseline_changes_and_emails() {
        let fixture = Fixture::new();
        fixture.write(PARENT_POLICY, PARENT_TEXT).write(PARENT_BASELINE, LOOSE);
        fixture.write(POLICY, OWN_TEXT).write(BASELINE, ONE).branch();
        fixture.write(POLICY, CHANGED).write(BASELINE, TWO).write(PARENT_POLICY, CHANGED).committed();
        let (root, policy, summary) = (fixture.path(CONSUMER), Policy::defaults(), fixture.beside());
        let admitted = |root, file| Baseline::admitted(root, &BaselineFile::from(file), policy.branch(Branch::Base));
        let announced = Surfaces::announce(&root, &policy, Some(&summary)).map(|()| fs::read_to_string(&summary));
        assert!(announced.is_ok_and(|listed| listed.is_ok_and(|listed| listed.ends_with(LISTED))));
        assert!(admitted(&root, PARENT_BASELINE).is_err() && admitted(fixture.root(), PARENT_BASELINE).is_ok());
        let linted = Lint::run(&root, &[], &policy, &mut Ledger::default());
        let prior = PriorPolicy::text_at_merge_base(fixture.root(), policy.branch(Branch::Base));
        assert!(matches!(linted, Err(Failure::Identity(_))) && prior.ok().flatten().as_deref() == Some(PARENT_TEXT));
    }
}
