use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use corpus_check::{Enforcement, LintName};
use crate_graph::CrateGraph;
use strum::IntoEnumIterator;
use task::{Amount, Branch, CommandLine, Ledger, Measure, Policy, Program, Report, Site, SourceFile, Stdout};
use test_budget::TestBudget;

use crate::task::failure::Failure;
use crate::task::ratchet::{Baseline, Drift, Ratchet, ViolationCount};
use crate::task::revision::Revision;

#[derive(Debug, Default)]
pub struct Lint {
    reports: BTreeMap<&'static str, Report>,
    listed: Vec<Report>,
}

impl Lint {
    const GIT: &str = "git";
    const PROSE_FILES: [&str; 7] = ["ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "*.md"];

    pub fn run(root: &Path, files: &[SourceFile], policy: &Policy, ledger: &mut Ledger) -> Result<(), Failure> {
        Revision::noreply_off(root, policy.branch(Branch::Base))?;
        let allowed = Baseline::admitted(root, policy.baseline(), policy.branch(Branch::Base))?;
        let gathered = Self::gathered(files, policy);
        gathered.listed.iter().for_each(|report| print!("{report}"));
        let observed = gathered.ratchet(&allowed, |_| true)?;
        println!("lint: {} rust files, {} baselined violations", files.len(), observed.total());
        ledger.spend(Measure::LintFindings, Amount::Baseline(policy.baseline().clone()));
        ledger.spend(Measure::TestLineShare, TestBudget::line_share(files, policy));
        ledger.spend(Measure::TangleItems, CrateGraph::largest_tangle(files));
        ledger.spend(Measure::EffectShare, Amount::Share(CrateGraph::effect_share(files)));
        ledger.spend(Measure::EffectDispersion, Amount::Count(CrateGraph::effect_dispersion(files)));
        Self::prose(root, policy, ledger)
    }

    pub fn shrink(root: &Path, files: &[SourceFile], policy: &Policy) -> Result<Baseline, Failure> {
        let path = policy.baseline().within(root);
        let observed = Self::gathered(files, policy).ratchet(&Baseline::read(&path)?, Drift::grew)?;
        observed.write(&path)?;
        Ok(observed)
    }

    fn prose(root: &Path, policy: &Policy, ledger: &mut Ledger) -> Result<(), Failure> {
        let git = CommandLine::new(Program::Executable(PathBuf::from(Self::GIT))).in_directory(root.to_path_buf());
        let listed = git.arguments(Self::PROSE_FILES).captured::<Stdout>()?;
        listed.split('\0').filter(|path| !path.is_empty()).try_for_each(|path| {
            let text = fs::read_to_string(root.join(path))
                .map_err(|error| task::Failure::Io { site: Site::File(root.join(path)), error })?;
            ledger.measure(PathBuf::from(path), &text, policy.spec());
            Ok::<(), Failure>(())
        })
    }

    fn gathered(files: &[SourceFile], policy: &Policy) -> Lint {
        let enforced =
            |enforcement: Enforcement| LintName::iter().filter(move |lint| lint.enforcement() == enforcement);
        Lint {
            reports: enforced(Enforcement::Ratchet)
                .map(|lint| (<&'static str>::from(lint), lint.report(files, policy)))
                .collect(),
            listed: enforced(Enforcement::Listed).map(|lint| lint.report(files, policy)).collect(),
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
    use task::{Ledger, Policy, Report, Site, SourceFile, Violation};

    const LINT_FINDINGS: &str = "lint-findings";
    const MEASURED_OK: &str = "-                     - ok";
    const PROSE_WORDS: &str = "prose-words ";
    const PROSE_SPENT: &str = " words                                6 ";
    const PROSE: &str = "a b c\n";
    const TRACKED: &str = "notes.md";
    const UNTRACKED: &str = "docs/more.md";
    const NOT_PROSE: &str = "notes.txt";
    const REPEATED: [(&str, &str); 3] = [
        ("crates/ant/src/lib.rs", "pub struct Ant; impl Ant { pub fn walk(&self) {} }"),
        ("crates/bee/src/lib.rs", "pub struct Bee; impl Bee { pub fn walk(&self) {} }"),
        ("crates/cow/src/lib.rs", "pub struct Cow; impl Cow { pub fn walk(&self) {} }"),
    ];
    const LISTED_LINTS: usize = 1;
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
    fn growth_over_the_baseline_fails_with_the_lint_report_shrinking_passes_and_listed_lints_stay_out() {
        let report = Report::new(HEADLINE, vec![Violation::new(Site::Workspace, "a comment")]);
        let lint = || Lint { reports: [(LINT, report.clone())].into(), ..Lint::default() };
        let refused = lint().ratchet(&Baseline::default(), |_| true).map_err(|failure| failure.to_string());
        assert_eq!(refused.err().as_deref(), Some(SHOWN));
        let allowed: Baseline = toml::from_str(ALLOWED).unwrap();
        assert!(Lint::default().ratchet(&allowed, Drift::grew).is_ok());
        let repeated = REPEATED.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        assert_eq!(Lint::gathered(&repeated, &Policy::defaults()).listed.len(), LISTED_LINTS);
    }

    #[test]
    fn a_baseline_above_the_findings_fails_until_shrunk_to_them_and_listed_findings_and_prose_do_not_ratchet() {
        let fixture = Fixture::new();
        fixture.write(BASELINE, ALLOWED).write(TRACKED, PROSE).branch();
        fixture.write(UNTRACKED, PROSE).write(NOT_PROSE, PROSE);
        let (policy, mut ledger) = (Policy::defaults(), Ledger::default());
        assert!(Lint::run(fixture.root(), &[], &policy, &mut ledger).is_err());
        Lint::shrink(fixture.root(), &[], &policy).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(BASELINE)).ok().as_deref(), Some(""));
        let repeated = REPEATED.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let passed = Lint::run(fixture.root(), &repeated, &policy, &mut ledger).is_ok();
        let statement = ledger.statement(&policy).to_string();
        let counted = statement.lines().any(|row| row.starts_with(PROSE_WORDS) && row.contains(PROSE_SPENT));
        assert!(
            passed
                && counted
                && statement.lines().any(|row| row.starts_with(LINT_FINDINGS) && row.ends_with(MEASURED_OK))
        );
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
