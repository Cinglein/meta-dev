use std::collections::BTreeMap;

use corpus_check::LintName;
use strum::IntoEnumIterator;
use task::{Policy, Report, SourceFile, Workspace};

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, Drift, Ratchet, ViolationCount};

#[derive(Debug)]
pub struct Lint {
    reports: BTreeMap<&'static str, Report>,
}

impl Lint {
    pub fn run(workspace: &Workspace, policy: &Policy) -> Result<(), Failure> {
        let files = workspace.source_files()?;
        let allowed = Baseline::admitted(workspace.root(), policy.baseline())?;
        let observed = Self::gathered(&files, policy).ratchet(&allowed, |_| true)?;
        println!("lint: {} rust files, {} baselined violations", files.len(), observed.total());
        Ok(())
    }

    pub fn shrink(workspace: &Workspace, policy: &Policy) -> Result<Baseline, Failure> {
        let path = policy.baseline().within(workspace.root());
        let observed =
            Self::gathered(&workspace.source_files()?, policy).ratchet(&Baseline::read(&path)?, Drift::grew)?;
        observed.write(&path)?;
        Ok(observed)
    }

    fn gathered(files: &[SourceFile], policy: &Policy) -> Lint {
        Lint {
            reports: LintName::iter().map(|lint| (<&'static str>::from(lint), lint.report(files, policy))).collect(),
        }
    }

    fn ratchet(self, allowed: &Baseline, failing: impl Fn(&Drift) -> bool) -> Result<Baseline, Failure> {
        let observed: Baseline =
            self.reports.iter().map(|(lint, report)| (*lint, ViolationCount::in_report(report))).collect();
        let drifts: Vec<Drift> = observed.drifts(allowed).into_iter().filter(|drift| failing(drift)).collect();
        if drifts.is_empty() { Ok(observed) } else { Err(Failure::Ratchet(Ratchet::new(drifts, self.reports))) }
    }
}

#[cfg(test)]
mod tests {
    use super::Lint;
    use crate::task::ratchet::{Baseline, Drift};
    use task::{Report, Site, Violation};

    const LINT: &str = "no-comments";
    const HEADLINE: &str = "comments go";
    const ALLOWED: &str = "no-comments = 1";
    const SHOWN: &str = "no-comments: 1 found, baseline 0\ncomments go:\nworkspace: a comment\n";

    #[test]
    fn growth_over_the_baseline_fails_with_the_lint_report_and_shrinking_passes() {
        let report = Report::new(HEADLINE, vec![Violation::new(Site::Workspace, "a comment")]);
        let lint = || Lint { reports: [(LINT, report.clone())].into() };
        let refused = lint().ratchet(&Baseline::default(), |_| true).map_err(|failure| failure.to_string());
        assert_eq!(refused.err().as_deref(), Some(SHOWN));
        let allowed: Baseline = toml::from_str(ALLOWED).unwrap();
        assert!(Lint { reports: [].into() }.ratchet(&allowed, Drift::grew).is_ok());
    }
}
