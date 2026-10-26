mod log_line;

use std::collections::BTreeSet;
use std::str::FromStr;

use crate::request::output::Listed;
use crate::request::project::LogWindow;
use log_line::LogLine;

#[derive(Clone, Debug)]
pub struct RawLog(Vec<LogLine>);

impl RawLog {
    pub fn excerpt(&self, window: &LogWindow) -> Listed {
        let lines = &self.0;
        let kept: BTreeSet<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.marks_failure().is_some())
            .flat_map(|(index, _)| window.index_span(index))
            .collect();
        let kept_lines: Vec<&LogLine> =
            kept.iter().take(window.limit().len()).filter_map(|index| lines.get(*index)).collect();
        let previous = std::iter::once(None).chain(kept_lines.iter().copied().map(Some));
        let shown: Vec<String> = kept_lines
            .iter()
            .zip(previous)
            .flat_map(|(line, previous)| {
                line.heading(previous).map(|heading| heading.to_string()).into_iter().chain([line.to_string()])
            })
            .collect();
        Listed::new(format!(
            "{}\n[kept {} of {} log lines; cut setup, passing output and anything beyond {}]\n",
            shown.join("\n"),
            kept_lines.len(),
            lines.len(),
            window.limit()
        ))
    }
}

impl FromStr for RawLog {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.lines().map(str::parse).collect::<Result<_, _>>()?))
    }
}

#[cfg(test)]
mod tests;
