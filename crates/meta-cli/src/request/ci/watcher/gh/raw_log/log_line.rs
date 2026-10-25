use std::fmt;
use std::str::FromStr;

use crate::request::output::Listed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLine {
    step: String,
    text: String,
}

impl LogLine {
    const MARKERS: &'static [&'static str] =
        &["##[error]", "panicked at", "assertion", "FAILED", "failures:", "found, baseline", "---- ", "Error:"];
    const ESCAPE: char = '\u{1b}';

    pub fn marks_failure(&self) -> Option<&Self> {
        let text = self.text.trim_start();
        (text.starts_with("error") || Self::MARKERS.iter().any(|marker| text.contains(marker))).then_some(self)
    }

    pub fn heading(&self, previous: Option<&Self>) -> Option<Listed> {
        previous.is_none_or(|previous| previous.step != self.step).then(|| Listed::new(format!("== {} ==", self.step)))
    }

    fn cleaned(raw: &str) -> String {
        let text = raw
            .split_once(' ')
            .filter(|(stamp, _)| {
                stamp.ends_with('Z') && stamp.contains('T') && stamp.starts_with(|c: char| c.is_ascii_digit())
            })
            .map_or(raw, |(_, rest)| rest);
        text.split_once(Self::ESCAPE).map_or_else(
            || text.to_owned(),
            |(head, tail)| format!("{head}{}", tail.split(Self::ESCAPE).map(Self::after_code).collect::<String>()),
        )
    }

    fn after_code(part: &str) -> &str {
        part.split_once('m').map_or(part, |(_, rest)| rest)
    }
}

impl FromStr for LogLine {
    type Err = std::convert::Infallible;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = line.split('\t').collect();
        Ok(match parts.as_slice() {
            [job, step, rest @ ..] => Self { step: format!("{job} / {step}"), text: Self::cleaned(&rest.join("\t")) },
            _ => Self { step: String::new(), text: Self::cleaned(line) },
        })
    }
}

impl fmt::Display for LogLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}
