mod prose_lines;

use std::fs;
use std::path::Path;

use task::{Report, Site, SpecSettings, Violation};

use crate::task::revision::Revision;
use prose_lines::ProseLines;

#[derive(Debug)]
pub struct SpecProse;

impl SpecProse {
    pub const LINT: &str = "spec-prose";

    pub fn since(root: &Path, spec: &SpecSettings, base: &Revision) -> Report {
        let written = fs::read_to_string(root.join(spec.file())).unwrap_or_default();
        Self::report(spec, &written, base.file(spec.file()).as_deref())
    }

    pub fn report(spec: &SpecSettings, written: &str, prior: Option<&str>) -> Report {
        let found = written.parse::<ProseLines>().unwrap_or_default();
        let grown = prior
            .and_then(|prior| prior.parse::<ProseLines>().ok())
            .filter(|allowed| found > *allowed)
            .map(|_| Violation::new(Site::File(spec.file().to_path_buf()), found));
        Report::new(
            "hand-written spec prose grew past the merge base; convert it to rule records",
            grown.into_iter().collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SpecProse;
    use task::SpecSettings;

    const SPEC: &str = "package = \"spec\"\nfile = \"context/SPEC.md\"\n";
    const UNMARKED: &str = "# Rules\n- kept\n";
    const ONE: &str = "# Rules\n# Not yet converted: hand-written prose, 1 lines\n\n- old\n";
    const TWO: &str = "# Not yet converted: hand-written prose, 2 lines\n- old\n\n- new\n";
    const GREW: &str = "context/SPEC.md: 2 unconverted lines";

    #[test]
    fn prose_may_shrink_or_hold_but_not_grow_past_a_marked_merge_base() {
        let spec: SpecSettings = toml::from_str(SPEC).unwrap();
        let report = |written, prior| SpecProse::report(&spec, written, prior);
        assert_eq!(report(TWO, Some(ONE)).violations().iter().map(ToString::to_string).collect::<Vec<_>>(), [GREW]);
        assert!(report(ONE, Some(TWO)).is_clean() && report(ONE, Some(ONE)).is_clean());
        assert!(report(TWO, Some(UNMARKED)).is_clean() && report(TWO, None).is_clean());
    }
}
