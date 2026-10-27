use std::fmt;

use chrono::{DateTime, Utc};

use crate::command_word::CommandWord;

#[derive(Debug)]
pub struct TraceFileName<'word> {
    at: DateTime<Utc>,
    word: &'word CommandWord,
}

impl<'word> TraceFileName<'word> {
    pub const EXTENSION: &'static str = "json";
    const STAMP: &'static str = "%Y%m%dT%H%M%S%.6fZ";

    pub fn new(at: DateTime<Utc>, word: &'word CommandWord) -> Self {
        Self { at, word }
    }
}

impl fmt::Display for TraceFileName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{}.{}", self.at.format(Self::STAMP), self.word, Self::EXTENSION)
    }
}
