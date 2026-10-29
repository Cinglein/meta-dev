use std::str::FromStr;

use meta_trace::{Trace, Verb};

use crate::request::cli_failure::{CliFailure, Misuse, Setup};
use crate::request::output::Listed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceView {
    Last(Option<Verb>),
    Chrome(Option<Verb>),
}

impl TraceView {
    pub fn verb(self) -> Option<Verb> {
        match self {
            Self::Last(verb) | Self::Chrome(verb) => verb,
        }
    }

    pub fn shown(self, trace: &Trace) -> Result<Listed, CliFailure> {
        match self {
            Self::Last(_) => Ok(Listed::new(trace)),
            Self::Chrome(_) => Ok(Listed::new(trace.chrome().map_err(CliFailure::raised::<Setup>)?).then("\n")),
        }
    }
}

impl FromStr for TraceView {
    type Err = CliFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let (view, verb) = words.split_first().ok_or_else(|| CliFailure::raised::<Misuse>("trace: last or chrome"))?;
        let verb = match verb {
            [] => Ok(None),
            [verb] => verb.parse().map(Some).map_err(|_| format!("unknown command `{verb}`")),
            _ => Err(format!("unknown trace command `{text}`")),
        };
        let verb = verb.map_err(CliFailure::raised::<Misuse>)?;
        match *view {
            "last" => Ok(Self::Last(verb)),
            "chrome" => Ok(Self::Chrome(verb)),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown trace command `{other}`"))),
        }
    }
}
