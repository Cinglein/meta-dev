use std::fmt;
use std::str::FromStr;

use task::SpecSettings;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProseLines(usize);

impl FromStr for ProseLines {
    type Err = ();

    fn from_str(text: &str) -> Result<ProseLines, ()> {
        text.split_once(SpecSettings::UNCONVERTED)
            .map(|(_, marked)| ProseLines(marked.lines().skip(1).filter(|line| !line.is_empty()).count()))
            .ok_or(())
    }
}

impl fmt::Display for ProseLines {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} unconverted lines", self.0)
    }
}
