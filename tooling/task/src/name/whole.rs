use super::matching::Matching;

#[derive(Debug)]
pub struct Whole;

impl Matching for Whole {
    fn holds(name: &str, word: &str) -> bool {
        name == word
    }
}
