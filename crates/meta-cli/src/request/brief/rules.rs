use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules(String);

impl Rules {
    pub const FILE: &'static str = "self/agent-rules.md";
    const VERSION_PREFIX: &'static str = "version: ";
}

impl FromStr for Rules {
    type Err = &'static str;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let numbered = text
            .lines()
            .next()
            .and_then(|line| line.strip_prefix(Self::VERSION_PREFIX))
            .is_some_and(|version| version.parse::<u32>().is_ok());
        numbered.then(|| Self(text.to_owned())).ok_or("first line must be `version: <number>`")
    }
}

impl fmt::Display for Rules {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0.trim_end())
    }
}
