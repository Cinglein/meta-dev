mod commit;
mod foreign_emails;
mod git_paths;
mod tracked_path;

use std::ffi::OsStr;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::{GitArgv, Port};
use task::BranchName;

use crate::task::failure::Failure;
pub use commit::Commit;
pub use foreign_emails::ForeignEmails;
pub use git_paths::GitPaths;
pub use tracked_path::TrackedPath;

#[derive(Debug)]
pub struct Revision {
    git: Port<GitArgv>,
    commit: Commit,
}

impl Revision {
    const FULL_TREE: &str = "--full-tree";
    const PATHS: &str = "--";
    const NO_OPERANDS: [&str; 0] = [];
    const PATIENCE: Duration = Duration::from_secs(30);

    pub fn merge_base(root: &Path, base: &BranchName) -> Result<Revision, Failure> {
        let git = Port::new(root);
        let found = Self::read(&git, GitArgv::MergeBase, [base]).map_err(|_| task::Failure::NoMergeBase)?;
        Ok(Revision { commit: found.parse()?, git })
    }

    pub fn in_database(database: &Path, commit: Commit) -> Revision {
        Revision { git: Port::new(database), commit }
    }

    pub fn present(&self, path: &Path) -> Result<Option<String>, Failure> {
        self.found(path, None, self.commit.object(path))
    }

    pub fn tracked(&self, path: TrackedPath) -> Result<Option<String>, Failure> {
        self.found(path.path(), Some(Self::FULL_TREE), self.commit.tracked(path))
    }

    fn found(&self, path: &Path, scope: Option<&str>, object: String) -> Result<Option<String>, Failure> {
        let named = [self.commit.as_ref(), OsStr::new(Self::PATHS), path.as_os_str()];
        let operands = scope.map(OsStr::new).into_iter().chain(named);
        let Ok(listed) = Self::read(&self.git, GitArgv::LsTree, operands)?.parse::<GitPaths>();
        (!listed.is_empty()).then(|| Self::read(&self.git, GitArgv::CatBlob, [object])).transpose()
    }

    pub fn changed_paths(&self) -> Result<GitPaths, Failure> {
        let Ok(listed) = Self::read(&self.git, GitArgv::ChangedNames, [&self.commit])?.parse::<GitPaths>();
        let Ok(untracked) = Self::read(&self.git, GitArgv::UntrackedNames, Self::NO_OPERANDS)?.parse::<GitPaths>();
        Ok(listed.and(untracked))
    }

    pub fn emails(root: &Path, base: &BranchName) -> Result<String, Failure> {
        Self::read(&Port::new(root), GitArgv::LogEmails, [base])
    }

    fn read<Word: AsRef<OsStr>>(
        git: &Port<GitArgv>,
        call: GitArgv,
        operands: impl IntoIterator<Item = Word>,
    ) -> Result<String, Failure> {
        let reported = git.reported(call, operands, Instant::now() + Self::PATIENCE).map_err(Failure::Git)?;
        Ok(reported.text().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::fs;

    use meta_golden::{GoldenFailure, Goldens};
    use meta_run::GitArgv;
    use settings::EmbeddedDefaults;

    use super::{Commit, ForeignEmails, GitPaths, Revision};
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
    const CHANGED: &str =
        "surfaces = [\"tooling/policy.toml\"]\nbaseline = \"lint-baseline.toml\"\nbit_boundary = []\n";
    const VIEWED: [GitArgv; 5] =
        [GitArgv::MergeBase, GitArgv::LsTree, GitArgv::ChangedNames, GitArgv::UntrackedNames, GitArgv::LogEmails];

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
    fn every_recorded_revision_read_parses_to_its_golden_view() -> Result<(), GoldenFailure> {
        let recordings = Goldens::workspace().recordings()?;
        let viewed = VIEWED.map(|call| {
            let recorded = recordings.iter().filter(|recording| recording.argv().calls(call));
            recorded.map(|recording| recording.viewed(&Revision::view(call, recording.out()))).collect::<Vec<_>>()
        });
        assert!(viewed.iter().all(|views| !views.is_empty()));
        viewed.into_iter().flatten().collect()
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
