use super::kind::Kind;
use crate::request::scale_policy::Position;

#[derive(Clone, Debug, Default)]
pub struct Segment {
    words: Vec<String>,
    kinds: Vec<Kind>,
}

impl Segment {
    pub fn lone(text: &str) -> Self {
        Self::default().appended(text.to_owned(), Kind::Literal)
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn kinds_from(&self, position: Position) -> &[Kind] {
        self.kinds.get(position.index()..).unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn appended(mut self, text: String, kind: Kind) -> Self {
        self.words.push(text);
        self.kinds.push(kind);
        self
    }
}
