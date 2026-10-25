use std::iter::Sum;
use std::ops::Add;
use std::path::{Path, PathBuf};

use crate::rule::report::Report;
use crate::source_file::SourceFile;
use crate::violation::Violation;

mod amount;
mod measurement;
mod support_modules;
mod test_scan;

use amount::Amount;
use measurement::Measurement;
use support_modules::SupportModules;
use test_scan::TestScan;

#[derive(Default)]
pub struct TestBudget {
    files: Amount,
    lines: Amount,
    tests: Amount,
    test_lines: Amount,
    violations: Vec<Violation>,
}

impl TestBudget {
    const MAX_TESTS_PER_FILE: Amount = Amount::new(3);
    const MAX_ASSERTIONS_PER_TEST: Amount = Amount::new(3);
    const MAX_LINES_PER_TEST: Amount = Amount::new(20);
    const MAX_LITERALS_PER_TEST: Amount = Amount::new(4);
    const MAX_INTEGER_LITERAL: Amount = Amount::new(64);
    const MAX_TESTS_PER_HUNDRED_FILES: Amount = Amount::new(40);
    const MAX_TEST_LINE_PERCENT: Amount = Amount::new(18);

    pub fn report(files: &[SourceFile]) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        files.iter().map(|file| Self::measure(file, &support)).sum::<Self>().summarise()
    }

    fn measure(file: &SourceFile, support: &[PathBuf]) -> Self {
        let lines = Amount::new(file.text().lines().count());
        let scanned = TestScan::budget(file.path(), file.syntax());
        Self {
            files: Amount::ONE,
            lines,
            test_lines: if support.iter().any(|path| path == Path::new(file.path())) {
                lines
            } else {
                scanned.test_lines
            },
            ..scanned
        }
    }

    fn summarise(self) -> Report {
        println!(
            "test budget: {} tests in {} files, {} of {} lines",
            self.tests, self.files, self.test_lines, self.lines
        );
        let global = [
            Measurement::new(
                "tests across the workspace",
                self.tests,
                self.files.percent(Self::MAX_TESTS_PER_HUNDRED_FILES),
            ),
            Measurement::new("test lines", self.test_lines, self.lines.percent(Self::MAX_TEST_LINE_PERCENT)),
        ]
        .into_iter()
        .filter_map(Measurement::violation);
        Report::new("test budget exceeded", self.violations.into_iter().chain(global).collect())
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
    use super::TestScan;

    const SOURCE: &str = "
#[cfg(test)]
mod tests {
    #[test]
    fn example() {
        let squares = [Square::A1, Square::H8];
        assert_eq!(squares.len(), 2);
        assert!(Bitboard::EMPTY.is_empty());
        assert_ne!(\"a\", \"b\");
        assert_eq!(seed(), 0x2545_F491);
    }

    #[test]
    fn property() {
        proptest!(|(count in 0..100usize, flag: bool)| {
            prop_assert!(count < 100 || flag);
        });
    }

    fn helper() {
        assert!(true);
    }
}
";
    const REPORTED: [&str; 5] = [
        "has 4 assertions",
        "has 6 literals",
        "integer literal 0x2545_F491",
        "integer literal 100usize",
        "assertion outside a test",
    ];

    #[test]
    fn counts_assertions_literals_and_variants_inside_macros() {
        let parsed = syn::parse_file(SOURCE).expect("valid rust");
        let joined: Vec<String> =
            TestScan::budget("example.rs", &parsed).violations.iter().map(ToString::to_string).collect();
        let joined = joined.join("\n");
        assert!(REPORTED.iter().all(|report| joined.contains(report)), "{joined}");
    }
}
