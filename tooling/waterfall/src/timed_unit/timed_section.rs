mod span;

use serde::Deserialize;

use crate::seconds::Seconds;
use crate::section::Section;
use span::Span;

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct TimedSection(Section, Span);

impl TimedSection {
    pub(super) fn spent_in(self, wanted: Section) -> Option<Seconds> {
        (self.0 == wanted).then(|| self.1.length())
    }
}
