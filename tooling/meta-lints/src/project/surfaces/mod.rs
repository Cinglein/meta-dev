mod changed;

use serde::Deserialize;

use crate::failure::Failure;
use crate::path_prefix::PathPrefix;
use crate::prior::Revision;

pub use changed::Changed;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Surfaces(Vec<PathPrefix>);

impl Surfaces {
    pub fn union(&self, prior: Surfaces) -> Surfaces {
        Surfaces(self.0.iter().cloned().chain(prior.0).collect())
    }

    pub fn changed(&self, revision: &Revision) -> Result<Changed, Failure> {
        let paths = revision.changed_paths()?;
        Ok(Changed::new(paths.into_iter().filter(|path| self.0.iter().any(|surface| path.is_under(surface))).collect()))
    }
}

#[cfg(test)]
mod tests {
    use crate::fixture::Fixture;

    const CONFIG: &str = "xtask.toml";
    const CONFIG_TEXT: &str =
        "surfaces = [\"xtask.toml\", \"lint-baseline.toml\"]\n[lints]\nbaseline = \"lint-baseline.toml\"\n";
    const NARROWED_CONFIG: &str = "surfaces = []\n[lints]\nbaseline = \"lint-baseline.toml\"\n";
    const BASELINE: &str = "lint-baseline.toml";
    const RAISED_BASELINE: &str = "[crate-graph]\n\"crates/a\" = 1\n";
    const SOURCE: &str = "crates/a/src/lib.rs";
    const CHANGED_SOURCE: &str = "pub struct Beta;\n";
    const LISTED: &str = "!! enforcement surfaces changed since the merge base:";

    impl Fixture {
        fn surfaced() -> Fixture {
            let fixture = Fixture::new();
            fixture.write(CONFIG, CONFIG_TEXT).write(BASELINE, "").write(SOURCE, "pub struct Alpha;\n").branch();
            fixture
        }

        fn changed(&self) -> String {
            self.project().changed_surfaces().map(|changed| changed.to_string()).unwrap_or_default()
        }
    }

    #[test]
    fn a_changed_surface_is_listed() {
        let fixture = Fixture::surfaced();
        fixture.write(BASELINE, RAISED_BASELINE);
        assert_eq!(fixture.changed(), format!("\n{LISTED}\n- {BASELINE}\n"));
    }

    #[test]
    fn other_changes_list_nothing() {
        let fixture = Fixture::surfaced();
        fixture.write(SOURCE, CHANGED_SOURCE);
        assert_eq!(fixture.changed(), format!("\n{LISTED}\n"));
    }

    #[test]
    fn a_narrowed_surface_list_still_lists_the_dropped_surfaces() {
        let fixture = Fixture::surfaced();
        fixture.write(CONFIG, NARROWED_CONFIG).write(BASELINE, RAISED_BASELINE);
        assert!(fixture.changed().contains(&format!("- {BASELINE}\n- {CONFIG}\n")));
    }
}
