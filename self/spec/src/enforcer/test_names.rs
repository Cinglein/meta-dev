use std::collections::BTreeSet;
use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug)]
pub struct TestNames(BTreeSet<String>);

impl TestNames {
    const TEST: &str = "#[test]";
    const IGNORE: &str = "#[ignore";
    const FN: &str = "fn ";
    const ITEM_ENDS: [char; 3] = ['{', '}', ';'];

    pub fn holds(&self, name: &str) -> bool {
        self.0.contains(name)
    }
}

impl FromStr for TestNames {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<TestNames, Infallible> {
        let heads = text.split(Self::FN).map(|before| before.rsplit(Self::ITEM_ENDS).next().unwrap_or_default());
        let tests = heads
            .zip(text.split(Self::FN).skip(1))
            .filter(|(head, _)| head.contains(Self::TEST) && !head.contains(Self::IGNORE));
        Ok(TestNames(tests.filter_map(|(_, named)| named.split_once('(')).map(|(name, _)| name.to_owned()).collect()))
    }
}
