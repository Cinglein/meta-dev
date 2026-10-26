use std::iter::Sum;
use std::ops::Add;
use std::path::PathBuf;

use task::Report;
use task::Site;
use task::SourceFile;
use task::SupportModules;
use task::Violation;
use task::{Amount, Percent};
use task::{Policy, Share};

mod file_count;
mod line_count;
mod measurement;
mod test_count;
mod test_scan;

use file_count::FileCount;
use line_count::LineCount;
use measurement::Measurement;
use test_count::TestCount;
use test_scan::TestScan;

#[derive(Debug, Default)]
pub struct TestBudget {
    files: FileCount,
    lines: LineCount,
    tests: TestCount,
    test_lines: LineCount,
    violations: Vec<Violation>,
}

impl TestBudget {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Self::measured(files, policy).summarise(policy)
    }

    #[must_use]
    pub fn line_share(files: &[SourceFile], policy: &Policy) -> Amount {
        let measured = Self::measured(files, policy);
        Amount::Share(Percent::rounded_up(measured.test_lines, measured.lines))
    }

    fn measured(files: &[SourceFile], policy: &Policy) -> Self {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        files
            .iter()
            .map(|file| Self::measure(file, &support, policy))
            .sum::<Self>()
    }

    fn measure(file: &SourceFile, support: &[PathBuf], policy: &Policy) -> Self {
        let lines = LineCount::from(file.text().lines().count());
        let scanned = TestScan::budget(file.path(), file.syntax(), policy);
        Self {
            files: FileCount::from(1),
            lines,
            test_lines: if support.contains(&file.path().to_owned()) {
                lines
            } else {
                scanned.test_lines
            },
            ..scanned
        }
    }

    fn summarise(self, policy: &Policy) -> Report {
        println!(
            "test budget: {} tests in {} files, {} of {} lines",
            self.tests, self.files, self.test_lines, self.lines
        );
        let global = [
            Measurement::new(
                "tests across the workspace",
                self.tests,
                self.files.tests_allowed(policy),
            )
            .overrun(),
            Measurement::new(
                "test lines",
                self.test_lines,
                policy.share(Share::TestLinePercent).applied_to(self.lines),
            )
            .overrun(),
        ]
        .into_iter()
        .flatten()
        .map(|overrun| Violation::new(Site::Workspace, overrun));
        Report::new(
            "test budget exceeded",
            self.violations.into_iter().chain(global).collect(),
        )
    }
}

impl Add for TestBudget {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            files: self.files + other.files,
            lines: self.lines + other.lines,
            tests: self.tests + other.tests,
            test_lines: self.test_lines + other.test_lines,
            violations: [self.violations, other.violations].concat(),
        }
    }
}

impl Sum for TestBudget {
    fn sum<I: Iterator<Item = Self>>(budgets: I) -> Self {
        budgets.fold(Self::default(), Self::add)
    }
}
