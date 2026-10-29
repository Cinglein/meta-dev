use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use task::SpecSettings;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProseWords(usize);

impl FromStr for ProseWords {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<ProseWords, Infallible> {
        let (rendered, unconverted) =
            text.split_once(SpecSettings::UNCONVERTED).map_or((text, ""), |(rendered, marked)| {
                (rendered, marked.split_once('\n').map_or("", |(_, prose)| prose))
            });
        let unattached = rendered.lines().filter(|line| !line.starts_with(SpecSettings::RULE));
        Ok(ProseWords(unattached.chain(unconverted.lines()).flat_map(str::split_whitespace).count()))
    }
}

impl fmt::Display for ProseWords {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} unattached words", self.0)
    }
}
