mod breach;
mod lexer;

use std::convert::Infallible;
use std::str::FromStr;

use breach::Breach;
use lexer::Lexer;

#[derive(Debug, Default)]
pub struct Tokens {
    segments: Vec<Vec<String>>,
    breach: Option<Breach>,
}

impl Tokens {
    pub fn segments(&self) -> impl Iterator<Item = &[String]> {
        self.segments.iter().map(Vec::as_slice)
    }

    pub fn breach(&self) -> Option<&'static str> {
        self.breach.map(Breach::reason)
    }

    fn extended(mut self, segment: Vec<String>) -> Self {
        self.segments.extend(Some(segment).filter(|words| !words.is_empty()));
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
