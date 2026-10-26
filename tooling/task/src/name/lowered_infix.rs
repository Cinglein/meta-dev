use super::matching::Matching;

#[derive(Debug)]
pub struct LoweredInfix;

impl Matching for LoweredInfix {
    fn holds(name: &str, word: &str) -> bool {
        name.to_lowercase().contains(word)
    }
}
