mod unconverted;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use task::Policy;

use crate::section::Section;
use crate::sections::SECTIONS;
use crate::test_fn::TestFn;
use unconverted::Unconverted;

#[derive(Debug)]
pub struct SpecView {
    sections: &'static [Section],
    unconverted: Unconverted,
}

impl SpecView {
    const HEADER: &str = "# Meta-development repo: spec (draft, started 2026-10-22)\n\nSections down to the next top-level heading are rendered from the rule records in `self/spec`; edit those, then run `cargo run -p meta-spec`. Each rule shows its id, division, ladder rung and enforcer.\n\n";
    const UNSET: &str = "no [spec] file in tooling/policy.toml";
    const STALE: &str = "the spec file differs from its render: run `cargo run -p meta-spec`";

    #[must_use]
    pub fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    pub fn file() -> io::Result<PathBuf> {
        let policy = Policy::read(&Self::root()).map_err(io::Error::other)?;
        policy.spec().map(|spec| Self::root().join(spec.file())).ok_or_else(|| io::Error::other(Self::UNSET))
    }

    pub fn written(file: &Path) -> io::Result<String> {
        fs::read_to_string(file)
    }

    pub fn ensure_rendered(file: &Path) -> io::Result<()> {
        let written = Self::written(file)?;
        (Self::over(&written, SECTIONS).to_string() == written)
            .then_some(())
            .ok_or_else(|| io::Error::other(Self::STALE))?;
        SECTIONS
            .iter()
            .flat_map(Section::rules)
            .filter_map(|rule| rule.enforcement().test())
            .try_for_each(TestFn::ensure_runs)
    }

    pub fn rewrite(file: &Path) -> io::Result<SpecView> {
        let view = Self::over(&Self::written(file)?, SECTIONS);
        fs::write(file, view.to_string()).map(|()| view)
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
    use crate::enforcement::Enforcement::Tracked;
    use crate::rule::Rule;
    use task::TopDivision::PerformanceAndCost;

    const PROSE: &str = "\n## Old\n- kept\n";
    const RULES: &[Rule] = &[Rule::new("an-id", PerformanceAndCost, "Says.", Tracked)];
    const SECTION: &[Section] = &[Section::new("New", RULES)];
    const SHOWN: &str = "## New\n- `an-id` (performance-and-cost, rung 1, no enforcer): Says.\n\n# Not yet converted: hand-written prose, 4 words\n\n## Old\n- kept\n";
    const SPEC: &str = "SPEC.md";

    #[test]
    fn the_checked_in_spec_passes_its_check_and_rule_ids_are_unique() {
        SpecView::ensure_rendered(&SpecView::file().unwrap()).unwrap();
        let rules: Vec<&Rule> = SECTIONS.iter().flat_map(Section::rules).collect();
        assert_eq!(rules.iter().map(|rule| rule.rule_id()).collect::<BTreeSet<_>>().len(), rules.len());
    }

    #[test]
    fn prose_after_the_marker_or_a_whole_unmarked_file_is_kept_and_counted() {
        let rendered = SpecView::over(PROSE, SECTION).to_string();
        assert_eq!(rendered.strip_prefix(SpecView::HEADER), Some(SHOWN));
        assert_eq!(SpecView::over(&rendered, SECTION).to_string(), rendered);
        assert_eq!(SECTION.iter().flat_map(Section::rules).count(), RULES.len());
    }

    #[test]
    fn a_rewrite_renders_the_records_over_the_kept_prose_and_a_missing_spec_fails() {
        let root = TempDir::new().unwrap();
        let file = root.path().join(SPEC);
        assert!(SpecView::rewrite(&file).is_err());
        fs::write(&file, PROSE).unwrap();
        assert!(SpecView::ensure_rendered(&file).is_err());
        SpecView::rewrite(&file).unwrap();
        assert!(SpecView::ensure_rendered(&file).is_ok());
    }
}
