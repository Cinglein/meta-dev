use std::collections::BTreeSet;

use crate::components::Components;
use crate::method_key::MethodKey;
use crate::places::Places;
use crate::reduction::Reduction;
use crate::signature::Signature;
use crate::source_path::SourcePath;
use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;

pub trait Located {
    fn places(&self, _: &Places) -> BTreeSet<SourcePath> {
        BTreeSet::new()
    }
}

impl Located for TypeKey {
    fn places(&self, places: &Places) -> BTreeSet<SourcePath> {
        places.of_type(self)
    }
}

impl Located for TraitKey {
    fn places(&self, places: &Places) -> BTreeSet<SourcePath> {
        places.of_trait(self)
    }
}

impl Located for Signature {}

impl Located for MethodKey {}

impl Located for Components {}

impl Located for Reduction<TraitKey> {}
