mod changed;

pub use changed::Changed;

use std::path::Path;

use task::{Branch, BranchName, Policy, Surface};

use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;

#[derive(Debug)]
pub struct Surfaces;

impl Surfaces {
    pub const SUMMARY: &str = "GITHUB_STEP_SUMMARY";

    pub fn announce(root: &Path, policy: &Policy, summary: Option<&Path>) -> Result<(), Failure> {
        Self::changed(root, policy.surfaces(), policy.branch(Branch::Base))?.announce(summary)
    }

    pub fn changed(root: &Path, current: &[Surface], base: &BranchName) -> Result<Changed, Failure> {
        let revision = Revision::merge_base(root, base)?;
        let surfaces: Vec<Surface> =
            current.iter().cloned().chain(PriorPolicy::read_at(&revision)?.surfaces()).collect();
        let paths = revision.changed_paths()?;
        Ok(Changed::new(paths.into_iter().filter(|path| surfaces.iter().any(|surface| surface.covers(path))).collect()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use settings::EmbeddedDefaults;

    use super::Surfaces;
    use crate::task::fixture::Fixture;
    use task::{Branch, Policy, Surface};

    const POLICY: &str = "tooling/policy.toml";
    const POLICY_TEXT: &str = "surfaces = [\"tooling/policy.toml\", \"lint-baseline.toml\"]\n";
    const NARROWED: &str = "surfaces = []\n";
    const BASELINE: &str = "lint-baseline.toml";
    const RAISED: &str = "crate-graph = 1\n";
    const SOURCE: &str = "crates/a/src/lib.rs";
    const LISTED: &str = "- lint-baseline.toml\n";
    const SUMMARY: &str = "summary.md";
    const UNTRACKED: &str = "tooling/new.toml";
    const BETA: &str = "pub struct Beta;\n";

    impl Fixture {
        fn surfaced() -> Fixture {
            let fixture = Fixture::new();
            fixture.write(POLICY, POLICY_TEXT).write(BASELINE, "").write(SOURCE, "pub struct Alpha;\n").branch();
            fixture
        }

        fn listed(&self, surfaces: &[&str]) -> String {
            let surfaces: Vec<Surface> = surfaces.iter().copied().map(Surface::from).collect();
            Surfaces::changed(self.root(), &surfaces, Policy::defaults().branch(Branch::Base))
                .map(|changed| changed.to_string())
                .unwrap_or_default()
        }
    }

    #[test]
    fn changed_untracked_and_dropped_surfaces_are_listed_in_the_step_summary_and_other_changes_list_nothing() {
        let [summarized, other, narrowed] = [(); 3].map(|()| Fixture::surfaced());
        summarized.write(BASELINE, RAISED).write(UNTRACKED, RAISED);
        Surfaces::announce(summarized.root(), &Policy::defaults(), Some(&summarized.path(SUMMARY))).unwrap();
        let written = fs::read_to_string(summarized.path(SUMMARY)).unwrap_or_default();
        assert!(written.ends_with(&format!("{LISTED}- {UNTRACKED}\n")));
        let dropped = narrowed.write(POLICY, NARROWED).write(BASELINE, RAISED).listed(&[]);
        let unlisted = other.write(SOURCE, BETA).listed(&[POLICY, BASELINE]);
        assert!(dropped.ends_with(&format!("{LISTED}- {POLICY}\n")) && unlisted.is_empty());
    }
}
