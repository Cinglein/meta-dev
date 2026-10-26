mod prose_words;

use std::fs;
use std::path::{Path, PathBuf};

use task::{Report, Site, SpecSettings, Violation};

use crate::task::prior_policy::PriorPolicy;
use crate::task::revision::Revision;
use prose_words::ProseWords;

#[derive(Debug)]
pub struct SpecProse;

impl SpecProse {
    pub const LINT: &str = "spec-prose";

    pub fn since(root: &Path, spec: &SpecSettings, base: &Revision) -> Report {
        let written = fs::read_to_string(root.join(spec.file())).unwrap_or_default();
        let prior = base.file(&Self::prior_file(&PriorPolicy::read_at(base), spec));
        Self::report(spec, &written, prior.as_deref())
    }

    pub fn report(spec: &SpecSettings, written: &str, prior: Option<&str>) -> Report {
        let Ok(found) = written.parse::<ProseWords>();
        let grown = prior
            .map(|prior| {
                let Ok(allowed) = prior.parse::<ProseWords>();
                allowed
            })
            .filter(|allowed| found > *allowed)
            .map(|_| Violation::new(Site::File(spec.file().to_path_buf()), found));
        Report::new(
            "hand-written spec prose grew past the merge base; convert it to rule records",
            grown.into_iter().collect(),
        )
    }

    fn prior_file(prior: &PriorPolicy, spec: &SpecSettings) -> PathBuf {
        prior.spec().unwrap_or(spec).file().to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::SpecProse;
    use crate::task::prior_policy::PriorPolicy;
    use task::SpecSettings;

    const SPEC: &str = "package = \"spec\"\nfile = \"context/SPEC.md\"\n";
    const MOVED: &str = "[spec]\npackage = \"spec\"\nfile = \"old/SPEC.md\"\n";
    const RULES: &str = "# Spec\n- `rule-id` (a, b): many many many words here\n";
    const ONE: &str =
        "# Spec\n- `rule-id` (a, b): words\n# Not yet converted: hand-written prose, 3 words\n\n- old prose\n";
    const LONGER: &str = "# Spec\n# Not yet converted: hand-written prose, 4 words\n- old prose grown\n";
    const GREW: &str = "context/SPEC.md: 6 unattached words";

    #[test]
    fn words_may_shrink_or_hold_but_not_grow_past_the_merge_base_even_within_a_line() {
        let spec: SpecSettings = toml::from_str(SPEC).unwrap();
        let report = |written, prior| SpecProse::report(&spec, written, prior);
        assert_eq!(report(LONGER, Some(ONE)).violations().iter().map(ToString::to_string).collect::<Vec<_>>(), [GREW]);
        assert!(
            report(ONE, Some(LONGER)).is_clean() && report(ONE, Some(ONE)).is_clean() && report(ONE, None).is_clean()
        );
        assert!(!report(LONGER, Some(RULES)).is_clean());
    }

    #[test]
    fn the_merge_base_spec_is_read_where_the_merge_base_policy_put_it() {
        let spec: SpecSettings = toml::from_str(SPEC).unwrap();
        let moved: PriorPolicy = toml::from_str(MOVED).unwrap();
        assert_eq!(SpecProse::prior_file(&moved, &spec), Path::new("old/SPEC.md"));
        assert_eq!(SpecProse::prior_file(&PriorPolicy::default(), &spec), spec.file());
    }
}
