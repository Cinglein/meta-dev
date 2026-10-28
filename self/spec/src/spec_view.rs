mod unconverted;

use meta_door::Door;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use task::{Policy, SpecSettings};

use crate::section::Section;
use crate::sections::SECTIONS;
use crate::wording::Wording;
use unconverted::Unconverted;

#[derive(Debug)]
pub struct SpecView {
    sections: &'static [Section],
    unconverted: Unconverted,
}

impl SpecView {
    const TITLE: Wording = Wording::new("# Meta-development repo: spec (draft, started 2026-10-22)");
    const INTRO: Wording = Wording::new(
        "Rendered from the rule records in `self/spec` down to the next top-level heading; edit those, then run `cargo run -p meta-spec`. Each rule shows id, division, rung and enforcer.",
    );
    const UNSET: &str = "no [spec] file in tooling/policy.toml";
    const STALE: &str = "the spec file differs from its render: run `cargo run -p meta-spec`";
    const MISSING: &str = "these rules name an enforcer that does not exist: ";

    #[must_use]
    pub fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    pub fn file() -> io::Result<PathBuf> {
        let policy = Policy::read(&Self::root()).map_err(io::Error::other)?;
        policy.spec().map(|spec| Self::root().join(spec.file())).ok_or_else(|| io::Error::other(Self::UNSET))
    }

    pub fn follow(file: &Path, argument: Option<&str>) -> io::Result<()> {
        match argument {
            Some(argument) if argument == SpecSettings::CHECK => {
                Self::ensure_rendered(file).and_then(|()| Self::ensure_enforced(SECTIONS, &Self::root()))
            }
            _ => Self::rewrite(file).map(drop),
        }
    }

    pub fn ensure_enforced(sections: &[Section], root: &Path) -> io::Result<()> {
        let missing: Vec<String> = sections
            .iter()
            .flat_map(Section::rules)
            .filter(|rule| rule.enforcement().lost(root).is_some())
            .map(|rule| rule.rule_id().to_string())
            .collect();
        let named = || io::Error::other(format!("{}{}", Self::MISSING, missing.join(", ")));
        missing.is_empty().then_some(()).ok_or_else(named)
    }

    pub fn ensure_rendered(file: &Path) -> io::Result<()> {
        let written = Door::read(file)?;
        (Self::over(&written, SECTIONS).to_string() == written)
            .then_some(())
            .ok_or_else(|| io::Error::other(Self::STALE))
    }

    pub fn rewrite(file: &Path) -> io::Result<SpecView> {
        let view = Self::over(&Door::read(file)?, SECTIONS);
        Door::write(file, view.to_string()).map(|()| view)
    }

    fn over(written: &str, sections: &'static [Section]) -> SpecView {
        let Ok(unconverted) = written.parse::<Unconverted>();
        SpecView { sections, unconverted }
    }
}

impl fmt::Display for SpecView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}\n", Self::TITLE)?;
        writeln!(formatter, "{}\n", Self::INTRO)?;
        self.sections.iter().try_for_each(|section| write!(formatter, "{section}"))?;
        write!(formatter, "{}", self.unconverted)
    }
}

#[cfg(test)]
mod tests {
    use meta_door::Door;
    use std::collections::BTreeSet;

    use tempfile::TempDir;

    use super::{SECTIONS, Section, SpecSettings, SpecView};
    use crate::enforcement::Enforcement::{Check, Tracked};
    use crate::enforcer::Enforcer;
    use crate::rule::{Rule, RuleId};
    use crate::wording::Wording;
    use task::TopDivision::PerformanceAndCost;

    const PROSE: &str = "\n## Old\n- kept\n";
    const RULES: &[Rule] = &[Rule::new(RuleId::new("an-id"), PerformanceAndCost, Wording::new("Says."), Tracked)];
    const SECTION: &[Section] = &[Section::new(Wording::new("New"), RULES)];
    const LOST: &[Rule] = &[Rule::new(
        RuleId::new("lost"),
        PerformanceAndCost,
        Wording::new("Says."),
        Check(Enforcer::new("lost.rs", "a"), &[]),
    )];
    const NAMED: &str = "does not exist: lost";
    const CHECK: Option<&str> = Some(SpecSettings::CHECK);
    const OTHER: Option<&str> = Some("render");
    const SHOWN: &str = "## New\n- `an-id` (performance-and-cost, rung 1, no enforcer): Says.\n\n# Not yet converted: hand-written prose, 4 words\n\n## Old\n- kept\n";
    const SPEC: &str = "SPEC.md";

    #[test]
    fn the_checked_in_spec_passes_its_check_rule_ids_are_unique_and_a_lost_enforcer_is_named() {
        SpecView::follow(&SpecView::file().unwrap(), CHECK).unwrap();
        let rules: Vec<&Rule> = SECTIONS.iter().flat_map(Section::rules).collect();
        assert_eq!(rules.iter().map(|rule| rule.rule_id()).collect::<BTreeSet<_>>().len(), rules.len());
        let lost = SpecView::ensure_enforced(&[Section::new(Wording::new("New"), LOST)], &SpecView::root());
        assert!(lost.unwrap_err().to_string().ends_with(NAMED));
    }

    #[test]
    fn prose_after_the_marker_or_a_whole_unmarked_file_is_kept_and_counted() {
        let rendered = SpecView::over(PROSE, SECTION).to_string();
        let header = format!("{}\n\n{}\n\n", SpecView::TITLE, SpecView::INTRO);
        assert_eq!(rendered.strip_prefix(&header), Some(SHOWN));
        assert_eq!(SpecView::over(&rendered, SECTION).to_string(), rendered);
        assert_eq!(SECTION.iter().flat_map(Section::rules).count(), RULES.len());
    }

    #[test]
    fn check_refuses_a_stale_spec_and_any_other_argument_rewrites_it() {
        let root = TempDir::new().unwrap();
        let file = root.path().join(SPEC);
        assert!(SpecView::follow(&file, None).is_err());
        Door::write(&file, PROSE).unwrap();
        assert!(SpecView::follow(&file, CHECK).is_err());
        SpecView::follow(&file, OTHER).unwrap();
        assert!(SpecView::follow(&file, CHECK).is_ok());
    }
}
