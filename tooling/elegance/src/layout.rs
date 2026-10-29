use std::collections::BTreeSet;

use crate::field::Field;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layout {
    named: BTreeSet<Field>,
    positional: BTreeSet<Field>,
}

impl Layout {
    #[must_use]
    pub fn new(named: BTreeSet<Field>, positional: BTreeSet<Field>) -> Layout {
        Layout { named, positional }
    }

    #[must_use]
    pub fn named(&self) -> &BTreeSet<Field> {
        &self.named
    }

    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.named.iter().chain(&self.positional)
    }
}
