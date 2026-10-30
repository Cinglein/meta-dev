use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Program(String);

impl Program {
    pub fn names(&self, word: &str) -> bool {
        let base = word.rsplit('/').next().unwrap_or(word);
        base == self.0 || base.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.') == self.0
    }
}
