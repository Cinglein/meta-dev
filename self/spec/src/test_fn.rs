mod test_listing;

use std::fmt;
use std::io;

use test_listing::TestListing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TestFn {
    package: &'static str,
    path: &'static str,
}

impl TestFn {
    #[must_use]
    pub const fn new(package: &'static str, path: &'static str) -> TestFn {
        TestFn { package, path }
    }

    pub fn ensure_runs(&self) -> io::Result<()> {
        let listed = TestListing::listed_by(self.package, TestListing::ALL).map_err(io::Error::other)?;
        let ignored = TestListing::listed_by(self.package, TestListing::IGNORED).map_err(io::Error::other)?;
        self.runs_in(&listed, &ignored)
            .then_some(())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, self.to_string()))
    }

    fn runs_in(&self, listed: &TestListing, ignored: &TestListing) -> bool {
        listed.names(self.path) && !ignored.names(self.path)
    }
}

impl fmt::Display for TestFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} in {}", self.path, self.package)
    }
}

#[cfg(test)]
mod tests {
    use super::{TestFn, TestListing};

    const LISTED: &str = "spec::tests::holds: test\nspec::tests::skipped: test\nspec::tests::bench: bench\n";
    const IGNORED: &str = "spec::tests::skipped: test\n";
    const PATHS: [&str; 4] = ["spec::tests::holds", "spec::tests::skipped", "spec::tests::bench", "holds"];
    const RUNS: [bool; 4] = [true, false, false, false];

    #[test]
    fn only_a_listed_test_that_is_not_ignored_runs() {
        let Ok(listed) = LISTED.parse::<TestListing>();
        let Ok(ignored) = IGNORED.parse::<TestListing>();
        assert_eq!(PATHS.map(|path| TestFn::new("spec", path).runs_in(&listed, &ignored)), RUNS);
    }
}
