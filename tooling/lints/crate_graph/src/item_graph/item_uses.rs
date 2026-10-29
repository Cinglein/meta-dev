use std::collections::BTreeSet;

use task::Name;

use crate::effect::Effect;

#[derive(Clone, Debug, Default)]
pub struct ItemUses {
    names: Vec<Name>,
    effects: BTreeSet<Effect>,
}

impl ItemUses {
    pub fn new(names: Vec<Name>, effects: BTreeSet<Effect>) -> ItemUses {
        ItemUses { names, effects }
    }

    pub fn names(&self) -> &[Name] {
        &self.names
    }

    pub fn effects(&self) -> &BTreeSet<Effect> {
        &self.effects
    }

    pub fn absorb(&mut self, other: ItemUses) {
        self.names.extend(other.names);
        self.effects.extend(other.effects);
    }
}
