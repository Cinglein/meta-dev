use std::collections::BTreeMap;

use strum::IntoEnumIterator;

use crate::baseline::Baseline;
use crate::lint_config::LintConfig;
use crate::rule::Rule;
use crate::rule::report::Report;
use crate::source_file::SourceFile;
use crate::violation::Violation;

#[derive(Clone, Debug)]
pub struct Findings(BTreeMap<&'static str, Report>);

impl Findings {
    pub fn collect(files: &[SourceFile], config: &LintConfig) -> Findings {
        Findings(Rule::iter().map(|rule| (rule.into(), rule.report(files, config))).collect())
    }

    pub fn observed(&self) -> Baseline {
        self.0
            .iter()
            .map(|(rule, report)| ((*rule).to_owned(), Baseline::counted(report.violations().iter())))
            .collect()
    }

    pub fn violations_in(&self, rule: &str, file: &str) -> Vec<&Violation> {
        self.0
            .get(rule)
            .map(|report| report.violations().iter().filter(|violation| violation.site().file() == file).collect())
            .unwrap_or_default()
    }

    pub fn headline(&self, rule: &str) -> Option<&'static str> {
        self.0.get(rule).map(Report::headline)
    }
}
