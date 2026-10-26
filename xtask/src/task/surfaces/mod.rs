mod changed;

pub use changed::Changed;

use std::path::Path;

use task::{Policy, Surface};

use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;

#[derive(Debug)]
pub struct Surfaces;

impl Surfaces {
    pub const SUMMARY: &str = "GITHUB_STEP_SUMMARY";

    pub fn announce(root: &Path, policy: &Policy, summary: Option<&Path>) -> Result<(), Failure> {
        Self::changed(root, policy.surfaces())?.announce(summary)
    }

    pub fn changed(root: &Path, current: &[Surface]) -> Result<Changed, Failure> {
        let revision = Revision::merge_base(root)?;
        let surfaces: Vec<Surface> =
            current.iter().cloned().chain(PriorPolicy::read_at(&revision).surfaces()).collect();
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
    use task::{Policy, Surface};

    const POLICY: &str = "tooling/policy.toml";
    const POLICY_TEXT: &str = "surfaces = [\"tooling/policy.toml\", \"lint-baseline.toml\"]\n";
    const NARROWED: &str = "surfaces = []\n";
    const BASELINE: &str = "lint-baseline.toml";
    const RAISED: &str = "crate-graph = 1\n";
    const SOURCE: &str = "crates/a/src/lib.rs";
    const LISTED: &str = "- lint-baseline.toml\n";
    const SUMMARY: &str = "summary.md";
    const UNTRACKED: &str = "tooling/new.toml";

    impl Fixture {
        fn surfaced() -> Fixture {
            let fixture = Fixture::new();
            fixture.write(POLICY, POLICY_TEXT).write(BASELINE, "").write(SOURCE, "pub struct Alpha;\n").branch();
            fixture
        }

        fn listed(&self, surfaces: &[&str]) -> String {
            let surfaces: Vec<Surface> = surfaces.iter().copied().map(Surface::from).collect();
            Surfaces::changed(self.root(), &surfaces).map(|changed| changed.to_string()).unwrap_or_default()
        }
    }

    #[test]
    fn changed_and_untracked_surfaces_are_written_to_the_step_summary() {
        let fixture = Fixture::surfaced();
        fixture.write(BASELINE, RAISED).write(UNTRACKED, RAISED);
        Surfaces::announce(fixture.root(), &Policy::defaults(), Some(&fixture.path(SUMMARY))).unwrap();
        let written = fs::read_to_string(fixture.path(SUMMARY)).unwrap_or_default();
        assert!(written.ends_with(&format!("{LISTED}- {UNTRACKED}\n")));
    }

    #[test]
    fn other_changes_list_nothing() {
        let fixture = Fixture::surfaced();
        fixture.write(SOURCE, "pub struct Beta;\n");
        assert_eq!(fixture.listed(&[POLICY, BASELINE]), "");
    }

    #[test]
    fn a_narrowed_surface_list_still_lists_the_dropped_surfaces() {
        let fixture = Fixture::surfaced();
        fixture.write(POLICY, NARROWED).write(BASELINE, RAISED);
        assert!(fixture.listed(&[]).ends_with(&format!("{LISTED}- {POLICY}\n")));
    }
}
