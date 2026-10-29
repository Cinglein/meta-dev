mod code;
mod level;
mod span;

use serde::Deserialize;
use task::FailureKind;

use code::Code;
use level::Level;
use span::Span;

#[derive(Debug, Deserialize)]
pub(super) struct Message {
    level: Level,
    code: Option<Code>,
    #[serde(default)]
    spans: Vec<Span>,
    rendered: Option<String>,
}

impl Message {
    pub(super) fn is_error(&self) -> bool {
        self.level == Level::Error && self.spans.iter().any(Span::is_primary)
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
