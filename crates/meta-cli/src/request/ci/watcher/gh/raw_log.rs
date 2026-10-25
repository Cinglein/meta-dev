mod log_line;

use std::collections::BTreeSet;
use std::fmt;

use crate::request::output::Listed;
use crate::request::project::LineCount;
use log_line::LogLine;

#[derive(Clone, Debug, Default)]
pub struct RawLog(String);

impl RawLog {
    pub fn new(text: String) -> Self {
        Self(text)
    }

    pub fn excerpt(&self, limit: LineCount, context: LineCount) -> Listed {
        let lines: Vec<LogLine> = self.0.lines().filter_map(|line| line.parse().ok()).collect();
        let kept: BTreeSet<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.marks_failure().is_some())
            .flat_map(|(index, _)| index..=index.saturating_add(context.len()))
            .filter(|index| *index < lines.len())
            .collect();
        let kept_lines: Vec<&LogLine> = kept.iter().take(limit.len()).filter_map(|index| lines.get(*index)).collect();
        let previous = std::iter::once(None).chain(kept_lines.iter().copied().map(Some));
        let shown: Vec<String> = kept_lines
            .iter()
            .zip(previous)
            .flat_map(|(line, previous)| {
                line.heading(previous).map(|heading| heading.to_string()).into_iter().chain([line.to_string()])
            })
            .collect();
        Listed::new(format!(
            "{}\n[kept {} of {} log lines; cut setup, passing output and anything beyond {limit}]",
            shown.join("\n"),
            kept_lines.len(),
            lines.len()
        ))
    }
}

impl fmt::Display for RawLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests;
