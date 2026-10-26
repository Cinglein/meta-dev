use std::collections::BTreeMap;
use std::path::Path;

use corpus_check::LintName;
use strum::IntoEnumIterator;
use task::{Branch, Policy, Report, SourceFile};

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, Drift, Ratchet, ViolationCount};

#[derive(Debug)]
pub struct Lint {
    reports: BTreeMap<&'static str, Report>,
}

impl Lint {
    pub fn run(root: &Path, files: &[SourceFile], policy: &Policy) -> Result<(), Failure> {
        let allowed = Baseline::admitted(root, policy.baseline(), policy.branch(Branch::Base))?;
        let observed = Self::gathered(files, policy).ratchet(&allowed, |_| true)?;
        println!("lint: {} rust files, {} baselined violations", files.len(), observed.total());
        Ok(())
    }

    pub fn shrink(root: &Path, files: &[SourceFile], policy: &Policy) -> Result<Baseline, Failure> {
        let path = policy.baseline().within(root);
        let observed = Self::gathered(files, policy).ratchet(&Baseline::read(&path)?, Drift::grew)?;
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
    use std::fs;

    use settings::EmbeddedDefaults;

    use super::Lint;
    use crate::task::fixture::Fixture;
    use crate::task::ratchet::{Baseline, Drift};
    use task::{Policy, Report, Site, Violation};

    const LINT: &str = "no-comments";
    const HEADLINE: &str = "comments go";
    const ALLOWED: &str = "no-comments = 1";
    const SHOWN: &str = "no-comments: 1 found, baseline 0\ncomments go:\nworkspace: a comment\n";
    const BASELINE: &str = "self/lint-baseline.toml";
    const NOREPLY: &str = "1+someone@users.noreply.github.com";
    const WEB_COMMITTER: &str = "noreply@github.com";
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
        let policy = Policy::defaults();
        assert!(Lint::run(fixture.root(), &[], &policy).is_err());
        Lint::shrink(fixture.root(), &[], &policy).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(BASELINE)).ok().as_deref(), Some(""));
        assert!(Lint::run(fixture.root(), &[], &policy).is_ok());
    }

    #[test]
    fn every_commit_since_the_merge_base_has_noreply_author_and_committer_emails() {
        let fixture = Fixture::new();
        fixture.write(BASELINE, "").branch();
        let policy = Policy::defaults();
        assert!(Lint::run(fixture.commit(NOREPLY, WEB_COMMITTER).root(), &[], &policy).is_ok());
        let refused = Lint::run(fixture.commit(AUTHOR, COMMITTER).root(), &[], &policy);
        assert_eq!(refused.map_err(|failure| failure.to_string()).err().as_deref(), Some(FOREIGN));
    }
}
