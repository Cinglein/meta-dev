use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProseLines(usize);

impl ProseLines {
    const MARKER: &str = "# Not yet converted";
}

impl FromStr for ProseLines {
    type Err = ();

    fn from_str(text: &str) -> Result<ProseLines, ()> {
        text.split_once(Self::MARKER)
            .map(|(_, marked)| ProseLines(marked.lines().skip(1).filter(|line| !line.is_empty()).count()))
            .ok_or(())
    }
}

impl fmt::Display for ProseLines {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} unconverted lines", self.0)
    }
}
