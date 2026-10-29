use std::path::PathBuf;

use serde::Deserialize;

use super::code::Code;
use super::level::Level;
use super::span::Span;

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

    pub(super) fn is_lint(&self) -> bool {
        self.code.as_ref().is_some_and(Code::is_lint)
    }

    pub(super) fn primary_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.spans.iter().filter(|span| span.is_primary()).map(Span::path)
    }

    pub(super) fn rendered(&self) -> Option<&str> {
        self.rendered.as_deref()
    }
}
