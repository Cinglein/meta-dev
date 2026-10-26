use std::fs;
use std::path::Path;

use task::{LineNumber, Report, Site, Violation};

#[derive(Debug)]
pub struct SpecProse;

impl SpecProse {
    pub const LINT: &str = "spec-prose";
    const PATH: &str = "context/SPEC.md";
    const UNCONVERTED: &str = "# Not yet converted";

    pub fn report(root: &Path) -> Report {
        let text = fs::read_to_string(root.join(Self::PATH)).unwrap_or_default();
        let unconverted = text
            .lines()
            .zip(1..)
            .skip_while(|(line, _)| !line.starts_with(Self::UNCONVERTED))
            .skip(1)
            .filter(|(line, _)| !line.is_empty())
            .map(|(_, number)| Violation::new(Site::Line(Self::PATH.into(), LineNumber::new(number)), "unconverted"));
        Report::new(
            "hand-written spec prose only shrinks; convert it to rule records in self/spec",
            unconverted.collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::SpecProse;
    use tempfile::TempDir;

    const SPEC: &str = "# Rules\n- `id`: kept\n# Not yet converted: 2 lines\n\n## Old\n- prose\n";
    const FOUND: &str = "context/SPEC.md:5: unconverted|context/SPEC.md:6: unconverted";

    #[test]
    fn only_non_blank_lines_after_the_marker_count_and_a_missing_spec_counts_none() {
        let root = TempDir::new().unwrap();
        assert!(SpecProse::report(root.path()).is_clean());
        fs::create_dir(root.path().join("context")).unwrap();
        fs::write(root.path().join("context/SPEC.md"), SPEC).unwrap();
        let found: Vec<String> = SpecProse::report(root.path()).violations().iter().map(ToString::to_string).collect();
        assert_eq!(found.join("|"), FOUND);
    }
}
