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
    const OPEN: &'static str = "(";
    const CLOSE: &'static str = ")";

    pub fn segments(&self) -> impl Iterator<Item = &[String]> {
        self.segments.iter().map(Segment::words)
    }

    pub fn parts(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter()
    }

    pub fn breach(&self) -> Option<&'static str> {
        self.breach.map(Breach::reason)
    }

    pub fn flattened(self, nested: &dyn Fn(&[String]) -> Option<String>) -> Self {
        let start = Self { segments: Vec::new(), breach: self.breach };
        self.segments.into_iter().fold(start, |flat, part| flat.followed_by(part, nested))
    }

    fn followed_by(self, part: Segment, nested: &dyn Fn(&[String]) -> Option<String>) -> Self {
        let inner =
            nested(part.words()).and_then(|text| text.parse::<Self>().ok()).map(|inner| inner.flattened(nested));
        match inner {
            None => self.extended(part),
            Some(inner) => {
                let fenced = self.extended(part).extended(Segment::lone(Self::OPEN));
                let inside = inner.segments.into_iter().fold(fenced, Self::extended);
                inside.extended(Segment::lone(Self::CLOSE)).flagged(inner.breach)
            }
        }
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
