mod spec_prose;

use std::collections::BTreeMap;
use std::path::Path;

use corpus_check::LintName;
use crate_graph::CrateGraph;
use strum::IntoEnumIterator;
use task::{Amount, Branch, Ledger, Measure, Policy, Report, SourceFile};
use test_budget::TestBudget;

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, Drift, Ratchet, ViolationCount};
use crate::task::revision::Revision;
pub use spec_prose::SpecProse;

#[derive(Debug)]
pub struct Lint {
    reports: BTreeMap<&'static str, Report>,
}

impl Lint {
    pub fn run(root: &Path, files: &[SourceFile], policy: &Policy, ledger: &mut Ledger) -> Result<(), Failure> {
        Revision::noreply_off(root, policy.branch(Branch::Base))?;
        let allowed = Baseline::admitted(root, policy.baseline(), policy.branch(Branch::Base))?;
        let base = || Revision::merge_base(root, policy.branch(Branch::Base));
        let spec = policy.spec().map(|spec| base().and_then(|base| SpecProse::since(root, spec, &base))).transpose()?;
        let observed = Self::gathered(files, policy, spec).ratchet(&allowed, |_| true)?;
        println!("lint: {} rust files, {} baselined violations", files.len(), observed.total());
        ledger.spend(Measure::LintFindings, Amount::Baseline(policy.baseline().clone()));
        ledger.spend(Measure::TestLineShare, TestBudget::line_share(files, policy));
        ledger.spend(Measure::TangleItems, CrateGraph::largest_tangle(files));
        Ok(())
    }

    pub fn shrink(root: &Path, files: &[SourceFile], policy: &Policy) -> Result<Baseline, Failure> {
        let path = policy.baseline().within(root);
        let observed = Self::gathered(files, policy, None).ratchet(&Baseline::read(&path)?, Drift::grew)?;
        observed.write(&path)?;
        Ok(observed)
    }

    fn gathered(files: &[SourceFile], policy: &Policy, spec: Option<Report>) -> Lint {
        Lint {
            reports: LintName::iter()
                .map(|lint| (<&'static str>::from(lint), lint.report(files, policy)))
                .chain(spec.map(|spec| (SpecProse::LINT, spec)))
                .collect(),
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
    use std::fs;

    use settings::EmbeddedDefaults;

    use super::Lint;
    use crate::task::fixture::Fixture;
    use crate::task::ratchet::{Baseline, Drift};
    use task::{Ledger, Policy, Report, Site, Violation};

    const LINT_FINDINGS: &str = "lint-findings";
    const MEASURED_OK: &str = "-                     - ok";
    const LINT: &str = "no-comments";
    const HEADLINE: &str = "comments go";
    const ALLOWED: &str = "no-comments = 1";
    const SHOWN: &str = "no-comments: 1 found, baseline 0\ncomments go:\nworkspace: a comment\n";
    const BASELINE: &str = "self/lint-baseline.toml";
    const NOREPLY: &str = "1+someone@users.noreply.github.com";
    const WEB_COMMITTER: &str = "noreply@github.com";
    const MAIN: &str = "main@example.com";
    const AUTHOR: &str = "author@example.com";
    const COMMITTER: &str = "committer@example.com";
    const FOREIGN: &str = "commit emails are not noreply: author@example.com, committer@example.com";

    #[test]
    fn growth_over_the_baseline_fails_with_the_lint_report_and_shrinking_passes() {
        let report = Report::new(HEADLINE, vec![Violation::new(Site::Workspace, "a comment")]);
        let lint = || Lint { reports: [(LINT, report.clone())].into() };
        let refused = lint().ratchet(&Baseline::default(), |_| true).map_err(|failure| failure.to_string());
        assert_eq!(refused.err().as_deref(), Some(SHOWN));
        let allowed: Baseline = toml::from_str(ALLOWED).unwrap();
        assert!(Lint { reports: [].into() }.ratchet(&allowed, Drift::grew).is_ok());
    }

    #[test]
    fn a_baseline_above_the_findings_fails_until_shrunk_to_them() {
        let fixture = Fixture::new();
        fixture.write(BASELINE, ALLOWED).branch();
        let (policy, mut ledger) = (Policy::defaults(), Ledger::default());
        assert!(Lint::run(fixture.root(), &[], &policy, &mut ledger).is_err());
        Lint::shrink(fixture.root(), &[], &policy).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(BASELINE)).ok().as_deref(), Some(""));
        let passed = Lint::run(fixture.root(), &[], &policy, &mut ledger).is_ok();
        let statement = ledger.statement(&policy).to_string();
        assert!(passed && statement.lines().any(|row| row.starts_with(LINT_FINDINGS) && row.ends_with(MEASURED_OK)));
    }

    #[test]
    fn every_commit_off_the_base_branch_has_noreply_author_and_committer_emails() {
        let fixture = Fixture::new();
        fixture.write(BASELINE, "").branch();
        fixture.criss_cross([MAIN, MAIN], [NOREPLY, WEB_COMMITTER]).commit([AUTHOR, COMMITTER]);
        let refused = Lint::run(fixture.root(), &[], &Policy::defaults(), &mut Ledger::default());
        assert_eq!(refused.map_err(|failure| failure.to_string()).err().as_deref(), Some(FOREIGN));
    }
}
