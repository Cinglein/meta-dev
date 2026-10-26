use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pattern(String);

impl Pattern {
    const ANY: &str = "*";

    #[must_use]
    pub fn everything() -> Pattern {
        Pattern(Self::ANY.to_owned())
    }

    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        self.0
            .strip_suffix(Self::ANY)
            .map_or(self.0 == name, |prefix| name.starts_with(prefix))
    }

    #[must_use]
    pub fn covers(&self, inner: &Pattern) -> bool {
        self.matches(inner.0.trim_end_matches(Self::ANY))
            && (self.0.ends_with(Self::ANY) || !inner.0.ends_with(Self::ANY))
    }

    #[must_use]
    pub fn is_covered_by(&self, outer: &[Pattern]) -> bool {
        outer.iter().any(|pattern| pattern.covers(self))
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    const PREFIX: &str = "meta_*";
    const EXACT: &str = "meta_log";
    const OTHER: &str = "task";

    #[test]
    fn a_starred_pattern_matches_and_covers_its_prefix_and_an_exact_one_only_itself() {
        let [prefix, exact, other] = [PREFIX, EXACT, OTHER].map(|text| Pattern(text.to_owned()));
        assert!(prefix.matches(EXACT) && exact.matches(EXACT) && !prefix.matches(OTHER));
        assert!(
            Pattern::everything().covers(&prefix) && prefix.covers(&exact) && exact.covers(&exact)
        );
        assert!(
            !exact.covers(&prefix)
                && !prefix.covers(&other)
                && !prefix.covers(&Pattern::everything())
        );
    }
}
