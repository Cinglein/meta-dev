use std::iter::Sum;
use std::ops::Add;
use std::path::PathBuf;

use task::Report;
use task::Site;
use task::SourceFile;
use task::SupportModules;
use task::Violation;
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
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        files
            .iter()
            .map(|file| Self::measure(file, &support, policy))
            .sum::<Self>()
            .summarise(policy)
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

#[cfg(test)]
mod tests {
    use super::TestBudget;
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const LIB: &str = "
#[cfg(test)] mod fixtures;
#[cfg(test)]
mod tests {
    #[test]
    fn example() { let squares = [Square::A1, Square::H8]; assert_eq!(squares.len(), 2); assert!(Bitboard::EMPTY.is_empty()); assert_ne!(\"a\", \"b\"); assert_eq!(seed(), 0x2545_F491); }
    #[test]
    fn property() { proptest!(|(count in 0..100usize, flag: bool)| { prop_assert!(count < 100 || flag); }); }
    #[test] fn edge() { assert!(true); assert!(true); assert_eq!(64, 64); }
    fn helper() -> Vec<u8> { assert!(true); vec![1] }
}
";
    const LONG_HEAD: &str = "
#[cfg(test)]
mod tests {
    #[test] fn one() {}
    #[test] fn two() {}
    #[test] fn three() {}
    #[test]
    fn long() {";
    const PADDING: usize = 22;
    const PATHS: [&str; 3] = [
        "crates/a/src/lib.rs", "crates/a/src/fixtures.rs", "crates/a/src/long.rs",
    ];
    const REPORTED: [&str; 10] = [
        "lib.rs:6: fn example has 4 assertions", "lib.rs:6: fn example has 6 literals",
        "lib.rs:6: integer literal 0x2545_F491", "lib.rs:8: integer literal 100usize",
        "lib.rs:8: integer literal 100 in", "lib.rs:10: assertion outside a test",
        "long.rs:8: fn long has 21 body lines", "long.rs: 4 tests, at most 3 allowed",
        "7 tests across the workspace, at most 1", "has 42 test lines, at most 7 allowed",
    ];

    #[test]
    fn measures_tests_files_and_lines_and_flags_every_budget_overrun() {
        let long = format!("{LONG_HEAD}{}    }}\n}}\n", "\n".repeat(PADDING));
        let texts = [
            LIB.to_owned(),
            "pub struct Fixture;\npub struct Other;".to_owned(),
            long,
        ];
        let files: Vec<SourceFile> = PATHS
            .into_iter()
            .zip(texts)
            .map(|(path, text)| SourceFile::parse(path, text).unwrap())
            .collect();
        let report = TestBudget::report(&files, &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
