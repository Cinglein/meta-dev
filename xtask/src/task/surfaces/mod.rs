mod changed;

pub use changed::Changed;

use std::path::Path;

use task::{Branch, BranchName, Policy, Surface};

use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::{GitPaths, Revision};

#[derive(Debug)]
pub struct Surfaces;

impl Surfaces {
    pub const SUMMARY: &str = "GITHUB_STEP_SUMMARY";

    pub fn announce(root: &Path, policy: &Policy, summary: Option<&Path>) -> Result<(), Failure> {
        Self::changed(root, policy.surfaces(), policy.branch(Branch::Base))?.announce(summary)
    }

    pub fn changed(root: &Path, current: &[Surface], base: &BranchName) -> Result<Changed, Failure> {
        let revision = Revision::merge_base(root, base)?;
        Ok(Self::listed(current, PriorPolicy::read_at(&revision)?, revision.changed_paths()?))
    }

    fn listed(current: &[Surface], prior: PriorPolicy, paths: GitPaths) -> Changed {
        let surfaces: Vec<Surface> = current.iter().cloned().chain(prior.surfaces()).collect();
        Changed::new(paths.into_iter().filter(|path| surfaces.iter().any(|surface| surface.covers(path))).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Surfaces;
    use crate::task::fixture::Fixture;
    use crate::task::prior_policy::PriorPolicy;
    use task::Surface;

    const SURFACES: [&str; 2] = ["tooling", "lint-baseline.toml"];
    const PRIOR: &str = "surfaces = [\"tooling\", \"lint-baseline.toml\"]\n";
    const NONE: &str = "";
    const CHANGED: &str = "lint-baseline.toml\0tooling/new.toml\0crates/a/src/lib.rs\0";
    const SOURCE: &str = "crates/a/src/lib.rs\0";
    const SUMMARY: &str = "summary.md";
    const LISTED: &str = "- lint-baseline.toml\n- tooling/new.toml\n";

    impl Surfaces {
        fn shown(current: &[&str], prior: &str, changed: &str) -> super::Changed {
            let current: Vec<Surface> = current.iter().copied().map(Surface::from).collect();
            let Ok(paths) = changed.parse();
            Self::listed(&current, toml::from_str::<PriorPolicy>(prior).unwrap(), paths)
        }
    }

    #[test]
    fn changed_untracked_and_dropped_surfaces_are_listed_in_the_step_summary_and_other_changes_list_nothing() {
        let fixture = Fixture::new();
        let [dropped, kept] = [Surfaces::shown(&[], PRIOR, CHANGED), Surfaces::shown(&SURFACES, NONE, CHANGED)];
        let unlisted = Surfaces::shown(&SURFACES, PRIOR, SOURCE);
        let written =
            dropped.announce(Some(&fixture.path(SUMMARY))).map(|()| fs::read_to_string(fixture.path(SUMMARY)));
        assert!(dropped.to_string() == kept.to_string() && unlisted.is_empty());
        assert!(written.is_ok_and(|written| written.is_ok_and(|written| written.ends_with(LISTED))));
    }
}
