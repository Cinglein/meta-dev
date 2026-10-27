use std::collections::BTreeSet;
use std::path::PathBuf;

use task::{Report, Site, Violation};

use super::checked_file::CheckedFile;

#[derive(Debug)]
pub struct SpecDrift;

impl SpecDrift {
    pub const LINT: &str = "spec-drift";

    pub fn report(written: &str, prior: Option<&str>, changed: &[PathBuf]) -> Report {
        let kept: BTreeSet<&str> = prior.map(|prior| prior.lines().collect()).unwrap_or_default();
        let enforcer = |line: &str| line.parse::<CheckedFile>().ok().and_then(|file| file.among(changed));
        let drifted = written
            .lines()
            .filter(|line| kept.contains(line))
            .filter_map(|line| enforcer(line).map(|file| Violation::new(Site::File(file), line)));
        Report::new(
            "an enforcer changed but the rule that names it did not; review that rule's record",
            drifted.collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CheckedFile, SpecDrift};

    const PROSE: &str = "- `unchecked` (a, rung 1, no enforcer): x";
    const PRIOR: &str = "- `kept` (a, rung 3, check a.rs fn a): x\n- `moved` (a, rung 3, check b.rs fn b): x\n";
    const WRITTEN: &str = "- `kept` (a, rung 3, check a.rs fn a): x\n- `moved` (a, rung 3, check b.rs fn c): x\n";
    const CHANGED: [&str; 3] = ["a.rs", "b.rs", "c.rs"];
    const SHOWN: [&str; 1] = ["a.rs: - `kept` (a, rung 3, check a.rs fn a): x"];

    #[test]
    fn a_changed_enforcer_fails_only_while_its_rule_record_is_unchanged_since_the_merge_base() {
        let changed = CHANGED.map(PathBuf::from);
        let report = |prior| SpecDrift::report(WRITTEN, prior, &changed);
        let shown = |prior| report(prior).violations().iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!([shown(Some(PRIOR)), shown(None)], [SHOWN.map(String::from).to_vec(), Vec::new()]);
        let named =
            WRITTEN.lines().chain([PROSE]).map(|line| line.parse::<CheckedFile>().is_ok_and(|file| file.named()));
        assert!(SpecDrift::report(WRITTEN, Some(PRIOR), &[]).is_clean() && named.eq([true, true, false]));
    }
}
