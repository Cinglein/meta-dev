mod spec_prose;

use std::collections::BTreeMap;
use std::path::Path;

use corpus_check::{Enforcement, LintName};
use crate_graph::{CrateGraph, GraphReport};
use strum::IntoEnumIterator;
use task::{Amount, Branch, Ledger, Measure, Most, Policy, Report, SourceFile};
use test_budget::TestBudget;

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, Drift, Ratchet, ViolationCount};
use crate::task::revision::Revision;
pub use spec_prose::SpecProse;

#[derive(Debug, Default)]
pub struct Lint {
    reports: BTreeMap<&'static str, Report>,
    listed: BTreeMap<LintName, Report>,
    graphs: GraphReport,
}

impl Lint {
    pub fn run(root: &Path, files: &[SourceFile], policy: &Policy, ledger: &mut Ledger) -> Result<(), Failure> {
        Revision::noreply_off(root, policy.branch(Branch::Base))?;
        let allowed = Baseline::admitted(root, policy.baseline(), policy.branch(Branch::Base))?;
        let base = || Revision::merge_base(root, policy.branch(Branch::Base));
        let spec = policy.spec().map(|spec| base().and_then(|base| SpecProse::since(root, spec, &base))).transpose()?;
        Self::gathered(files, policy, spec).checked(files, &allowed, policy, ledger)
    }

    fn checked(
        self,
        files: &[SourceFile],
        allowed: &Baseline,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<(), Failure> {
        self.listed.values().for_each(|report| print!("{report}"));
        let repeated = self.listed.get(&LintName::RepeatedMethods).map(|report| report.violations().len());
        ledger.spend(Measure::TangleItems, self.graphs.largest_tangle());
        ledger.spend(Measure::EffectShare, Amount::Share(self.graphs.effect_share()));
        ledger.spend(Measure::EffectDispersion, Amount::Count(self.graphs.effect_dispersion()));
        ledger.spend(Measure::RepeatedShapes, Amount::Count(Most::from(repeated.unwrap_or_default())));
        let observed = self.ratchet(allowed, |_| true)?;
        println!("lint: {} rust files, {} baselined violations", files.len(), observed.total());
        ledger.spend(Measure::LintFindings, Amount::Baseline(policy.baseline().clone()));
        ledger.spend(Measure::TestLineShare, TestBudget::line_share(files, policy));
        Ok(())
    }

    pub fn shrink(root: &Path, files: &[SourceFile], policy: &Policy) -> Result<Baseline, Failure> {
        let path = policy.baseline().within(root);
        let observed = Self::gathered(files, policy, None).ratchet(&Baseline::read(&path)?, Drift::grew)?;
        observed.write(&path)?;
        Ok(observed)
    }

    fn gathered(files: &[SourceFile], policy: &Policy, spec: Option<Report>) -> Lint {
        let graphs = CrateGraph::graphs(files);
        let enforced =
            |enforcement: Enforcement| LintName::iter().filter(move |lint| lint.enforcement() == enforcement);
        Lint {
            reports: enforced(Enforcement::Ratchet)
                .map(|lint| (<&'static str>::from(lint), lint.report(files, &graphs, policy)))
                .chain(spec.map(|spec| (SpecProse::LINT, spec)))
                .collect(),
            listed: enforced(Enforcement::Listed).map(|lint| (lint, lint.report(files, &graphs, policy))).collect(),
            graphs,
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
    use crate::task::revision::ForeignEmails;
    use task::{Ledger, Measure, Policy, Report, Site, SourceFile, Violation};

    const LINT_FINDINGS: &str = "lint-findings";
    const MEASURED_OK: &str = "-                     - ok";
    const LINT: &str = "no-comments";
    const HEADLINE: &str = "comments go";
    const ALLOWED: &str = "no-comments = 1";
    const SHOWN: &str = "no-comments: 1 found, baseline 0\ncomments go:\nworkspace: a comment\n";
    const BASELINE: &str = "self/lint-baseline.toml";
    const LISTED: &str = "repeated-methods";
    const REPEATED: [(&str, &str); 3] = [
        ("crates/ant/src/lib.rs", "pub struct Ant; impl Ant { pub fn walk(&self) {} }"),
        ("crates/bee/src/lib.rs", "pub struct Bee; impl Bee { pub fn walk(&self) {} }"),
        ("crates/cow/src/lib.rs", "pub struct Cow; impl Cow { pub fn walk(&self) { std::process::exit(CODE) } }"),
    ];
    const RECORDED: [(Measure, &str); 4] = [
        (Measure::TangleItems, "0"),
        (Measure::EffectShare, "100%"),
        (Measure::EffectDispersion, "1"),
        (Measure::RepeatedShapes, "1"),
    ];
    const NOREPLY: &str = "1+someone@users.noreply.github.com\nnoreply@github.com\n";
    const EMAILS: &str =
        "1+someone@users.noreply.github.com\nnoreply@github.com\ncommitter@example.com\nauthor@example.com\n";
    const FOREIGN: &str = "commit emails are not noreply: author@example.com, committer@example.com";

    #[test]
    fn growth_over_the_baseline_fails_with_the_lint_report_and_shrinking_passes() {
        let report = Report::new(HEADLINE, vec![Violation::new(Site::Workspace, "a comment")]);
        let lint = || Lint { reports: [(LINT, report.clone())].into(), ..Lint::default() };
        let refused = lint().ratchet(&Baseline::default(), |_| true).map_err(|failure| failure.to_string());
        assert_eq!(refused.err().as_deref(), Some(SHOWN));
        let allowed: Baseline = toml::from_str(ALLOWED).unwrap();
        assert!(Lint::default().ratchet(&allowed, Drift::grew).is_ok());
        let triple = REPEATED.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let gathered = Lint::gathered(&triple, &Policy::defaults(), None);
        assert!(gathered.listed.len() == 1 && !gathered.reports.contains_key(LISTED));
    }

    #[test]
    fn a_baseline_above_the_findings_fails_until_shrunk_to_them_and_every_lint_measure_is_recorded() {
        let fixture = Fixture::new();
        fixture.write(BASELINE, ALLOWED);
        let (policy, mut ledger) = (Policy::defaults(), Ledger::default());
        let files = REPEATED.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let checked = |ledger: &mut Ledger| {
            let allowed = Baseline::read(&fixture.path(BASELINE))?;
            Lint::gathered(&files, &policy, None).checked(&files, &allowed, &policy, ledger)
        };
        assert!(checked(&mut ledger).is_err() && Lint::shrink(fixture.root(), &files, &policy).is_ok());
        let passed = checked(&mut ledger).is_ok()
            && fs::read_to_string(fixture.path(BASELINE)).is_ok_and(|text| text.is_empty());
        let statement = ledger.statement(&policy).to_string();
        assert!(passed && statement.lines().any(|row| row.starts_with(LINT_FINDINGS) && row.ends_with(MEASURED_OK)));
        let recorded = RECORDED.map(|(measure, _)| ledger.spent(&measure).map(ToString::to_string));
        assert_eq!(recorded, RECORDED.map(|(_, amount)| Some(amount.to_owned())));
    }

    #[test]
    fn every_commit_off_the_base_branch_has_noreply_author_and_committer_emails() {
        let refused = ForeignEmails::refused(EMAILS).map_err(|failure| failure.to_string());
        assert!(refused.err().as_deref() == Some(FOREIGN) && ForeignEmails::refused(NOREPLY).is_ok());
    }
}
