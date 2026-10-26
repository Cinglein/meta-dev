use std::fmt;
use std::str::FromStr;

use task::Failure;

pub(super) struct ListedTest(String);

impl ListedTest {
    const SUFFIX: &str = ": test";
}

impl fmt::Display for ListedTest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ListedTest {
    type Err = Failure;

    fn from_str(line: &str) -> Result<ListedTest, Failure> {
        line.strip_suffix(Self::SUFFIX)
            .map(|name| ListedTest(name.to_owned()))
            .ok_or_else(|| Failure::Command(format!("{line} is not a listed test")))
    }
}

#[cfg(test)]
mod tests {
    use super::ListedTest;
    #[test]
    fn libtest_lists_a_test_as_its_name_and_the_word_test_and_benches_are_not_tests() {
        let listed: ListedTest = "tests::one: test".parse().unwrap();
        let bench = "tests::one: bench".parse::<ListedTest>().is_err();
        assert!(listed.to_string() == "tests::one" && bench);
    }
}
