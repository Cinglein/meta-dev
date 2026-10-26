use std::collections::BTreeSet;
use std::convert::Infallible;
use std::str::FromStr;

use task::{CargoArgument, CommandLine, Program, Stdout};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TestListing(BTreeSet<String>);

impl TestListing {
    pub const ALL: &[CargoArgument] = &[CargoArgument::List];
    pub const IGNORED: &[CargoArgument] = &[CargoArgument::List, CargoArgument::Ignored];
    const LISTED: &str = ": test";

    pub fn listed_by(package: &str, listing: &[CargoArgument]) -> Result<TestListing, task::Failure> {
        let Ok(listed) = CommandLine::new(Program::Cargo)
            .arguments([CargoArgument::Test, CargoArgument::Package])
            .argument(package)
            .argument(CargoArgument::ToolArguments)
            .arguments(listing)
            .captured::<Stdout>()?
            .parse();
        Ok(listed)
    }

    #[must_use]
    pub fn names(&self, path: &str) -> bool {
        self.0.contains(path)
    }
}

impl FromStr for TestListing {
    type Err = Infallible;

    fn from_str(listing: &str) -> Result<TestListing, Infallible> {
        Ok(TestListing(listing.lines().filter_map(|line| line.strip_suffix(Self::LISTED)).map(str::to_owned).collect()))
    }
}
