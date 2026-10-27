use std::collections::BTreeSet;
use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug)]
pub struct TestNames(BTreeSet<String>);

impl TestNames {
    const TEST: &str = "#[test]";
    const FN: &str = "fn ";

    pub fn holds(&self, name: &str) -> bool {
        self.0.contains(name)
    }
}

impl FromStr for TestNames {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<TestNames, Infallible> {
        let declared = text.split(Self::TEST).skip(1).filter_map(|after| after.trim_start().strip_prefix(Self::FN));
        Ok(TestNames(declared.filter_map(|named| named.split_once('(')).map(|(name, _)| name.to_owned()).collect()))
    }
}
