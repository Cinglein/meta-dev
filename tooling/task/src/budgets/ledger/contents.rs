mod spec_prose;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::budgets::Amount;
use crate::budgets::word_count::WordCount;
use crate::domains::Domain;
use crate::policy::{Most, SpecSettings};
use spec_prose::SpecProse;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Contents(BTreeMap<PathBuf, Most>);

impl Contents {
    pub fn hold(&mut self, path: PathBuf, text: &str, spec: Option<&SpecSettings>) {
        let words = spec.filter(|spec| spec.file() == path).map_or_else(
            || {
                let Ok(count) = text.parse::<WordCount>();
                count.most()
            },
            |_| {
                let Ok(prose) = text.parse::<SpecProse>();
                prose.most()
            },
        );
        self.0.insert(path, words);
    }

    #[must_use]
    pub fn words(&self, domain: &Domain) -> Amount {
        let held = self.0.iter().filter(|(path, _)| domain.holds(path));
        Amount::Count(Most::from(
            held.map(|(_, words)| words.count()).sum::<usize>(),
        ))
    }
}

#[cfg(test)]
mod tests;
