use std::convert::Infallible;
use std::str::FromStr;

use meta_schema::Text;

use crate::policy::Most;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordCount(Most);

impl Default for WordCount {
    fn default() -> WordCount {
        WordCount(Most::from(0))
    }
}

impl WordCount {
    #[must_use]
    pub fn most(self) -> Most {
        self.0
    }
}

impl FromStr for WordCount {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<WordCount, Infallible> {
        Ok(WordCount(Most::from(text.split_whitespace().count())))
    }
}

impl From<&Text> for WordCount {
    fn from(text: &Text) -> WordCount {
        let Ok(count) = text.to_string().parse();
        count
    }
}
