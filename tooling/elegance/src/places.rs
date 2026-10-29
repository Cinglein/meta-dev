use std::collections::{BTreeMap, BTreeSet};

use crate::source_path::SourcePath;
use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Places {
    types: BTreeMap<TypeKey, SourcePath>,
    traits: BTreeMap<TraitKey, SourcePath>,
}

impl Places {
    pub fn type_at(&mut self, owner: Option<TypeKey>, place: Option<SourcePath>) {
        self.types.extend(owner.zip(place));
    }

    pub fn trait_at(&mut self, key: Option<TraitKey>, place: Option<SourcePath>) {
        self.traits.extend(key.zip(place));
    }

    #[must_use]
    pub fn of_type(&self, owner: &TypeKey) -> BTreeSet<SourcePath> {
        self.types.get(owner).cloned().into_iter().collect()
    }

    #[must_use]
    pub fn of_trait(&self, key: &TraitKey) -> BTreeSet<SourcePath> {
        self.traits.get(key).cloned().into_iter().collect()
    }
}
