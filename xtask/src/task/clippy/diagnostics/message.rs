mod code;
mod level;
mod span;
mod standing;

use serde::Deserialize;
use task::FailureKind;

use code::Code;
use level::Level;
use span::Span;
pub(in crate::task::clippy::diagnostics) use standing::Standing;

#[derive(Debug, Deserialize)]
pub(super) struct Message {
    level: Level,
    code: Option<Code>,
    #[serde(default)]
    spans: Vec<Span>,
    #[serde(default, rename = "message")]
    text: String,
    rendered: Option<String>,
}

impl Message {
    const SUMMARIES: [&'static str; 2] = ["aborting due to", "could not compile"];

    pub(super) fn standing(&self) -> Option<Standing> {
        (self.level != Level::Other).then(|| match (self.is_placed(), self.summary().is_some()) {
            (true, _) => Standing::Placed,
            (false, true) => Standing::Summary,
            (false, false) => Standing::Unplaced,
        })
    }

    fn is_placed(&self) -> bool {
        self.spans.iter().any(Span::is_primary)
    }

    fn summary(&self) -> Option<&'static str> {
        Self::SUMMARIES.into_iter().find(|summary| self.text.starts_with(summary))
    }

    pub(super) fn failure(&self) -> FailureKind {
        let paths = self.spans.iter().filter(|span| span.is_primary()).map(Span::path).collect();
        match &self.code {
            Some(code) => code.failure(paths),
            None => FailureKind::Compile { paths },
        }
    }

    pub(super) fn rendered(&self) -> Option<&str> {
        self.rendered.as_deref()
    }
}
