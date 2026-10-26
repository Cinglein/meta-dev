mod unconverted;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::section::Section;
use crate::sections::SECTIONS;
use unconverted::Unconverted;

#[derive(Debug)]
pub struct SpecView {
    sections: &'static [Section],
    unconverted: Unconverted,
}

impl SpecView {
    const PATH: &str = "context/SPEC.md";
    const HEADER: &str = "# Meta-development repo: spec (draft, started 2026-10-22)\n\nSections down to the next top-level heading are rendered from the rule records in `self/spec`; edit those, then run `cargo run -p meta-spec`. Each rule shows its id, division, ladder rung and enforcer.\n\n";

    #[must_use]
    pub fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    pub fn written(root: &Path) -> io::Result<String> {
        fs::read_to_string(root.join(Self::PATH))
    }

    pub fn rewrite(root: &Path) -> io::Result<()> {
        Self::written(root)
            .and_then(|written| fs::write(root.join(Self::PATH), Self::over(&written, SECTIONS).to_string()))
    }

    fn over(written: &str, sections: &'static [Section]) -> SpecView {
        let Ok(unconverted) = written.parse::<Unconverted>();
        SpecView { sections, unconverted }
    }
}

impl fmt::Display for SpecView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(Self::HEADER)?;
        self.sections.iter().try_for_each(|section| write!(formatter, "{section}"))?;
        write!(formatter, "{}", self.unconverted)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;

    use tempfile::TempDir;

    use super::{SECTIONS, Section, SpecView};
    use crate::division::Division::PerformanceAndCost;
    use crate::enforcement::Enforcement::Tracked;
    use crate::rule::Rule;

    const PROSE: &str = "\n## Old\n- kept\n";
    const RULES: &[Rule] = &[Rule::new("an-id", PerformanceAndCost, "Says.", Tracked)];
    const SECTION: &[Section] = &[Section::new("New", RULES)];
    const SHOWN: &str = "## New\n- `an-id` (performance-and-cost, rung 1, no enforcer): Says.\n\n# Not yet converted: hand-written prose, 2 lines\n\n## Old\n- kept\n";
    const STALE: &str = "context/SPEC.md differs from its render: run `cargo run -p meta-spec`";

    #[test]
    fn the_checked_in_spec_is_its_render_with_unique_ids_and_existing_test_enforcers() {
        let written = SpecView::written(&SpecView::root()).unwrap();
        assert!(SpecView::over(&written, SECTIONS).to_string() == written, "{STALE}");
        let rules: Vec<&Rule> = SECTIONS.iter().flat_map(Section::rules).collect();
        assert_eq!(rules.iter().map(|rule| rule.rule_id()).collect::<BTreeSet<_>>().len(), rules.len());
        assert!(
            rules.iter().filter_map(|rule| rule.enforcement().test()).all(|test| test.exists_under(&SpecView::root()))
        );
    }

    #[test]
    fn prose_after_the_marker_or_a_whole_unmarked_file_is_kept_and_counted() {
        let rendered = SpecView::over(PROSE, SECTION).to_string();
        assert_eq!(rendered.strip_prefix(SpecView::HEADER), Some(SHOWN));
        assert_eq!(SpecView::over(&rendered, SECTION).to_string(), rendered);
    }

    #[test]
    fn a_rewrite_renders_the_records_over_the_kept_prose_and_a_missing_spec_fails() {
        let root = TempDir::new().unwrap();
        assert!(SpecView::rewrite(root.path()).is_err());
        fs::create_dir(root.path().join("context")).unwrap();
        fs::write(root.path().join(SpecView::PATH), PROSE).unwrap();
        SpecView::rewrite(root.path()).unwrap();
        assert_eq!(SpecView::written(root.path()).ok(), Some(SpecView::over(PROSE, SECTIONS).to_string()));
    }
}
