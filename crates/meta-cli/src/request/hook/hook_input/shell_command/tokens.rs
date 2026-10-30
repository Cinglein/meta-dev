mod breach;
mod kind;
mod lexer;
mod segment;

use std::convert::Infallible;
use std::str::FromStr;

use breach::Breach;
pub use kind::Kind;
use lexer::Lexer;
pub use segment::Segment;

#[derive(Debug, Default)]
pub struct Tokens {
    segments: Vec<Segment>,
    breach: Option<Breach>,
}

impl Tokens {
    pub fn segments(&self) -> impl Iterator<Item = &[String]> {
        self.segments.iter().map(Segment::words)
    }

    pub fn parts(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter()
    }

    pub fn breach(&self) -> Option<&'static str> {
        self.breach.map(Breach::reason)
    }

    fn extended(mut self, segment: Segment) -> Self {
        self.segments.extend(Some(segment).filter(|segment| !segment.is_empty()));
        self
    }

    fn flagged(self, breach: Option<Breach>) -> Self {
        Self { breach: self.breach.or(breach), ..self }
    }
}

impl FromStr for Tokens {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = text.chars().collect();
        let lexer = (0..chars.len())
            .fold(Lexer::default(), |lexer, index| lexer.stepped(chars.get(index..).unwrap_or_default()));
        Ok(lexer.finished())
    }
}
