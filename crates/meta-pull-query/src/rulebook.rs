use std::collections::HashMap;

use meta_pulls::{BaseBranch, RequiredChecks};

#[derive(Clone, Debug, Default)]
pub struct Rulebook(HashMap<BaseBranch, RequiredChecks>);

impl Rulebook {
    #[must_use]
    pub fn required(&self, base: &BaseBranch) -> Option<&RequiredChecks> {
        self.0.get(base)
    }
}

impl FromIterator<(BaseBranch, RequiredChecks)> for Rulebook {
    fn from_iter<Rules: IntoIterator<Item = (BaseBranch, RequiredChecks)>>(rules: Rules) -> Self {
        Self(rules.into_iter().fold(HashMap::new(), |mut book, (base, required)| {
            let united = book.remove(&base).unwrap_or_default().united(required);
            book.insert(base, united);
            book
        }))
    }
}
