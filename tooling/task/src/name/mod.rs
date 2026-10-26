mod casing;
mod lowered_infix;
mod matching;
mod prefix;
mod whole;

pub use casing::Casing;
pub use lowered_infix::LoweredInfix;
pub use matching::Matching;
pub use prefix::Prefix;
pub use whole::Whole;

use std::fmt;
use std::path::Path;

use syn::Ident;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    #[must_use]
    pub fn length(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn matches<Rule: Matching>(&self, words: &[&str]) -> bool {
        words.iter().any(|word| Rule::holds(&self.0, word))
    }

    #[must_use]
    pub fn casing(&self) -> Casing {
        let first = self.0.chars().next();
        if first.is_some_and(|letter| letter.is_ascii_lowercase()) {
            Casing::Lower
        } else if first.is_some_and(|letter| letter.is_ascii_uppercase()) {
            if self
                .0
                .chars()
                .any(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit())
            {
                Casing::Camel
            } else {
                Casing::Shouting
            }
        } else {
            Casing::Other
        }
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Name {
        Name(ident.to_string())
    }
}

impl From<&str> for Name {
    fn from(text: &str) -> Name {
        Name(text.to_owned())
    }
}

impl PartialEq<&str> for Name {
    fn eq(&self, text: &&str) -> bool {
        self.0 == *text
    }
}

impl PartialEq<String> for Name {
    fn eq(&self, text: &String) -> bool {
        self.0 == *text
    }
}

impl PartialEq<Ident> for Name {
    fn eq(&self, ident: &Ident) -> bool {
        *ident == self.0
    }
}

impl AsRef<Path> for Name {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Casing, LoweredInfix, Name, Prefix, Whole};
    const NAMES: [&str; 4] = ["is_empty", "TableSize", "FOO", "_hidden"];
    const CASINGS: [Casing; 4] = [
        Casing::Lower,
        Casing::Camel,
        Casing::Shouting,
        Casing::Other,
    ];
    const PREFIX: [&str; 1] = ["is_"];
    const INFIX: [&str; 1] = ["size"];

    #[test]
    fn casing_reads_the_first_letter_and_matching_rules_differ_by_where_the_word_sits() {
        let names = NAMES.map(Name::from);
        assert_eq!(names.clone().map(|name| name.casing()), CASINGS);
        assert_eq!(
            names.clone().map(|name| name.matches::<Prefix>(&PREFIX)),
            names.clone().map(|name| name.casing() == Casing::Lower)
        );
        assert!(
            names
                .iter()
                .any(|name| name.matches::<LoweredInfix>(&INFIX))
                && !names.iter().any(|name| name.matches::<Whole>(&INFIX))
        );
    }
}
