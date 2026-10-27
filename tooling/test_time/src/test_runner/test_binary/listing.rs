use std::str::FromStr;

use super::TestBinary;
use super::listed_test::ListedTest;
use crate::test_run::TestRun;
use task::Failure;

#[derive(Debug)]
pub(super) enum Listing {
    Libtest(Vec<ListedTest>),
    Custom,
}

impl Listing {
    const SUMMARY: [&str; 2] = [" benchmarks", " benchmark"];

    pub(super) fn runs(self, binary: &TestBinary) -> Vec<TestRun> {
        match self {
            Listing::Libtest(tests) => tests.into_iter().map(|test| binary.listed(test)).collect(),
            Listing::Custom => vec![binary.whole()],
        }
    }
}

impl FromStr for Listing {
    type Err = Failure;

    fn from_str(listed: &str) -> Result<Listing, Failure> {
        let summary = listed
            .lines()
            .next_back()
            .filter(|last| Self::SUMMARY.iter().any(|ending| last.ends_with(ending)));
        Ok(summary.map_or(Listing::Custom, |_| {
            Listing::Libtest(
                listed
                    .lines()
                    .filter_map(|line| line.parse().ok())
                    .collect(),
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::Listing;

    const LISTED: [&str; 3] = ["a: test\n1 test, 0 benchmarks", "0 tests, 0 benchmarks", ""];

    #[test]
    fn a_libtest_listing_ends_in_its_summary_and_anything_else_is_one_custom_binary() {
        let [one, none, custom] = LISTED.map(|listed| listed.parse::<Listing>().ok());
        let libtest = matches!(one, Some(Listing::Libtest(tests)) if tests.len() == 1);
        let empty = matches!(none, Some(Listing::Libtest(tests)) if tests.is_empty());
        assert!(libtest && empty && matches!(custom, Some(Listing::Custom)));
    }
}
