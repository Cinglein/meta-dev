mod commit;

use std::path::{Path, PathBuf};

use task::{BranchName, CommandLine, Program, Stdout};

use crate::task::failure::Failure;
use commit::Commit;

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

    fn git(root: &Path) -> CommandLine {
        CommandLine::new(Program::Executable(PathBuf::from(Self::GIT))).in_directory(root.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use crate::task::fixture::Fixture;
    use crate::task::ratchet::Baseline;
    use crate::task::surfaces::Surfaces;
    use task::{BaselineFile, BranchName, Surface};

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
}
