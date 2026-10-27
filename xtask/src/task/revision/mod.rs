mod commit;
mod foreign_emails;

use std::path::{Path, PathBuf};

use task::{BranchName, CommandLine, Program, Stdout};

use crate::task::failure::Failure;
pub use commit::Commit;
pub use foreign_emails::ForeignEmails;

#[derive(Debug)]
pub struct Revision {
    root: PathBuf,
    commit: Commit,
}

impl Revision {
    const GIT: &str = "git";

    pub fn merge_base(root: &Path, base: &BranchName) -> Result<Revision, Failure> {
        let found = Self::git(root).arguments(["merge-base", "HEAD"]).argument(base).captured::<Stdout>();
        let commit = found.map_err(|_| task::Failure::NoMergeBase)?.parse()?;
        Ok(Revision { root: root.to_path_buf(), commit })
    }

    pub fn file(&self, path: &Path) -> Option<String> {
        Self::git(&self.root)
            .arguments(["cat-file", "blob"])
            .argument(self.commit.object(path))
            .captured::<Stdout>()
            .ok()
    }

    pub fn present(&self, path: &Path) -> Result<Option<String>, Failure> {
        let listed = Self::git(&self.root)
            .arguments(["ls-tree", "-z", "--name-only"])
            .argument(&self.commit)
            .arguments(["--"])
            .argument(path.display())
            .captured::<Stdout>()?;
        if listed.is_empty() {
            Ok(None)
        } else {
            let blob = Self::git(&self.root).arguments(["cat-file", "blob"]).argument(self.commit.object(path));
            Ok(Some(blob.captured::<Stdout>()?))
        }
    }

    pub fn changed_paths(&self) -> Result<Vec<PathBuf>, Failure> {
        let listed = Self::git(&self.root)
            .arguments(["diff", "--name-only", "--no-renames", "--relative", "-z"])
            .argument(&self.commit)
            .captured::<Stdout>()?;
        let untracked = Self::git(&self.root)
            .arguments(["ls-files", "--others", "--exclude-standard", "-z"])
            .captured::<Stdout>()?;
        Ok([listed, untracked]
            .iter()
            .flat_map(|paths| paths.split('\0'))
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .collect())
    }

    pub fn noreply_off(root: &Path, base: &BranchName) -> Result<(), Failure> {
        let emails = Self::git(root)
            .arguments(["log", "--format=%ae%n%ce", "HEAD", "--not"])
            .argument(base)
            .captured::<Stdout>()?;
        ForeignEmails::refused(&emails)
    }

    fn git(root: &Path) -> CommandLine {
        CommandLine::new(Program::Executable(PathBuf::from(Self::GIT))).in_directory(root.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Revision;
    use crate::task::fixture::Fixture;
    use crate::task::lint::SpecProse;
    use crate::task::prior_policy::PriorPolicy;
    use crate::task::ratchet::Baseline;
    use crate::task::surfaces::Surfaces;
    use task::{BaselineFile, BranchName, SpecSettings, Surface};

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
    const BASE: &str = "origin/main";
    const LOST: &str = "0123abc";
    const ABSENT: &str = "absent.toml";
    const SPEC: &str = "package = \"spec\"\nfile = \"context/SPEC.md\"\n";
    const CHANGED: &str =
        "surfaces = [\"tooling/policy.toml\"]\nbaseline = \"lint-baseline.toml\"\nbit_boundary = []\n";

    #[test]
    fn a_workspace_below_the_git_top_level_reads_its_own_prior_policy_baseline_and_changes() {
        let fixture = Fixture::new();
        fixture.write(PARENT_POLICY, PARENT_TEXT).write(PARENT_BASELINE, LOOSE);
        fixture.write(POLICY, OWN_TEXT).write(BASELINE, ONE).branch();
        fixture.write(POLICY, CHANGED).write(BASELINE, TWO);
        let root = fixture.path(CONSUMER);
        let surfaces = [Surface::from(PARENT_POLICY)];
        let Ok(base) = BASE.parse::<BranchName>();
        let listed = Surfaces::changed(&root, &surfaces, &base).map(|changed| changed.to_string()).unwrap_or_default();
        assert!(listed.ends_with(LISTED));
        assert!(Baseline::admitted(&root, &BaselineFile::from(PARENT_BASELINE), &base).is_err());
    }

    #[test]
    fn a_file_absent_at_the_revision_is_none_and_a_git_failure_is_an_error() {
        let fixture = Fixture::new();
        fixture.write(PARENT_POLICY, PARENT_TEXT).branch();
        let Ok(base) = BASE.parse::<BranchName>();
        let tip = Revision::merge_base(fixture.root(), &base).unwrap();
        let lost = Revision { root: fixture.root().to_path_buf(), commit: LOST.parse().unwrap() };
        let read = |revision: &Revision, path: &str| revision.present(Path::new(path)).map(|text| text.is_some()).ok();
        let found = [read(&tip, PARENT_POLICY), read(&tip, ABSENT), read(&lost, PARENT_POLICY)];
        assert_eq!(found, [Some(true), Some(false), None]);
    }

    #[test]
    fn a_git_failure_reading_the_prior_policy_or_spec_fails_closed() {
        let fixture = Fixture::new();
        fixture.write(PARENT_POLICY, PARENT_TEXT).branch();
        let lost = Revision { root: fixture.root().to_path_buf(), commit: LOST.parse().unwrap() };
        let spec: SpecSettings = toml::from_str(SPEC).unwrap();
        assert!(PriorPolicy::read_at(&lost).is_err() && SpecProse::since(fixture.root(), &spec, &lost).is_err());
    }
}
