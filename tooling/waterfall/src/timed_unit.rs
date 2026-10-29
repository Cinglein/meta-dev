mod timed_section;

use std::cmp::Ordering;
use std::fmt;

use serde::Deserialize;

use crate::seconds::Seconds;
use crate::section::Section;
use task::CrateName;
use timed_section::TimedSection;

#[derive(Debug, Deserialize)]
pub(crate) struct TimedUnit {
    #[serde(rename = "i")]
    index: usize,
    name: CrateName,
    target: String,
    start: Seconds,
    duration: Seconds,
    #[serde(rename = "unblocked_units")]
    unblocked_index: Vec<usize>,
    #[serde(rename = "unblocked_rmeta_units")]
    unblocked_rmeta_index: Vec<usize>,
    sections: Option<Vec<TimedSection>>,
}

impl TimedUnit {
    pub(crate) fn crate_name(&self) -> &CrateName {
        &self.name
    }

    pub(crate) fn end(&self) -> Seconds {
        self.start + self.duration
    }

    pub(crate) fn ended(&self, other: &TimedUnit) -> Ordering {
        self.end().ordered(other.end())
    }

    pub(crate) fn unblocked(&self, unit: &TimedUnit) -> bool {
        self.unblocked_index.iter().chain(&self.unblocked_rmeta_index).any(|index| *index == unit.index)
    }

    pub(crate) fn section(&self, wanted: Section) -> Seconds {
        match wanted {
            Section::Whole => self.duration,
            Section::Frontend | Section::Codegen | Section::Other => {
                self.sections.iter().flatten().filter_map(|section| section.spent_in(wanted)).sum()
            }
        }
    }
}

impl fmt::Display for TimedUnit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}{} at {} took {} (frontend {}, codegen {})",
            self.name,
            self.target,
            self.start,
            self.duration,
            self.section(Section::Frontend),
            self.section(Section::Codegen)
        )
    }
}
