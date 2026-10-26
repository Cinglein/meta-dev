mod prose_lines;

use std::fs;
use std::path::Path;

use task::{Report, Site, Violation};

use crate::task::revision::Revision;
use prose_lines::ProseLines;

#[derive(Debug)]
pub struct SpecProse;

impl SpecProse {
    pub const LINT: &str = "spec-prose";
    const PATH: &str = "context/SPEC.md";

    pub fn since(root: &Path, base: &Revision) -> Report {
        let written = fs::read_to_string(root.join(Self::PATH)).unwrap_or_default();
        Self::report(&written, base.file(Path::new(Self::PATH)).as_deref())
    }

    pub fn report(written: &str, prior: Option<&str>) -> Report {
        let found = written.parse::<ProseLines>().unwrap_or_default();
        let grown = prior
            .and_then(|prior| prior.parse::<ProseLines>().ok())
            .filter(|allowed| found > *allowed)
            .map(|_| Violation::new(Site::File(Self::PATH.into()), found));
        Report::new(
            "hand-written spec prose grew past the merge base; convert it to rule records in self/spec",
            grown.into_iter().collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SpecProse;

    const UNMARKED: &str = "# Rules\n- kept\n";
    const ONE: &str = "# Rules\n# Not yet converted\n\n- old\n";
    const TWO: &str = "# Not yet converted\n- old\n\n- new\n";
    const GREW: &str = "context/SPEC.md: 2 unconverted lines";

    #[test]
    fn prose_may_shrink_or_hold_but_not_grow_past_a_marked_merge_base() {
        let shown: Vec<String> =
            SpecProse::report(TWO, Some(ONE)).violations().iter().map(ToString::to_string).collect();
        assert_eq!(shown, [GREW]);
        assert!(SpecProse::report(ONE, Some(TWO)).is_clean() && SpecProse::report(ONE, Some(ONE)).is_clean());
        assert!(SpecProse::report(TWO, Some(UNMARKED)).is_clean() && SpecProse::report(TWO, None).is_clean());
    }
}
