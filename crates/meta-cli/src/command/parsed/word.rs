use std::fmt;

#[derive(Clone, Debug)]
pub struct Word(String);

impl Word {
    pub fn new(word: impl fmt::Display) -> Self {
        Self(word.to_string())
    }
}

impl fmt::Display for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
