use std::convert::Infallible;
use std::str::FromStr;

use crate::task::adoption::skip::Skip;

#[derive(Debug, Default)]
pub struct ShellLine(Vec<String>);

impl ShellLine {
    const CALL: [&str; 2] = ["cargo", "xtask"];
    const FALLBACK: &str = "||";
    const PIPE: &str = "|";
    const OPEN: &str = "${{";
    const CLOSE: &str = "}}";

    pub fn calls(&self, task: &str) -> bool {
        matches!(self.0.as_slice(), [cargo, xtask, called, ..] if [cargo.as_str(), xtask.as_str()] == Self::CALL && called == task)
    }

    pub fn skip(&self) -> Option<Skip> {
        let fallback = self.0.iter().any(|word| word.contains(Self::FALLBACK)).then_some(Skip::Fallback);
        fallback.or_else(|| self.0.iter().any(|word| word.contains(Self::PIPE)).then_some(Skip::Unpiped))
    }
}

impl FromStr for ShellLine {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<ShellLine, Infallible> {
        let words = |text: &str| text.split_whitespace().map(str::to_owned).collect::<Vec<String>>();
        Ok(ShellLine(line.split_once(Self::OPEN).map_or_else(
            || words(line),
            |(before, expression)| {
                let after = expression.split_once(Self::CLOSE).map_or("", |(_, after)| after);
                let Ok(ShellLine(rest)) = after.parse::<ShellLine>();
                words(before).into_iter().chain(rest).collect()
            },
        )))
    }
}
