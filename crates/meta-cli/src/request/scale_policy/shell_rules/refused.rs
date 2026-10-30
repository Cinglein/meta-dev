use std::fmt;

#[derive(Debug)]
pub struct Refused {
    words: Vec<String>,
}

impl Refused {
    pub fn spelled(words: &[&str]) -> Self {
        Self { words: words.iter().map(|word| (*word).to_owned()).collect() }
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.words.join(" "))
    }
}
