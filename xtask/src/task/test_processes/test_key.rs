use std::convert::Infallible;
use std::str::FromStr;

use task::{CrateName, Selection};

#[derive(Debug)]
pub struct TestKey(CrateName);

impl TestKey {
    pub fn selected(&self, selection: &Selection) -> bool {
        selection.within([&self.0]).reaching().is_some()
    }
}

impl FromStr for TestKey {
    type Err = Infallible;

    fn from_str(key: &str) -> Result<TestKey, Infallible> {
        let package = key.split_once(' ').map_or(key, |(package, _)| package);
        package.parse().map(TestKey)
    }
}
