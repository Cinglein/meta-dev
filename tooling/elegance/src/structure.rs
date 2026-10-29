use std::collections::BTreeMap;

use crate::layout::Layout;
use crate::method_use::MethodUse;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Structure {
    layouts: BTreeMap<TypeKey, Layout>,
    methods: Vec<MethodUse>,
}

impl Structure {
    pub fn laid_out(&mut self, owner: Option<TypeKey>, layout: Layout) {
        self.layouts.extend(owner.map(|owner| (owner, layout)));
    }

    pub fn used(&mut self, methods: impl IntoIterator<Item = MethodUse>) {
        self.methods.extend(methods);
    }

    #[must_use]
    pub fn layouts(&self) -> &BTreeMap<TypeKey, Layout> {
        &self.layouts
    }

    #[must_use]
    pub fn layout(&self, owner: &TypeKey) -> Option<&Layout> {
        self.layouts.get(owner)
    }

    pub fn methods_of<'structure>(
        &'structure self,
        owner: &'structure TypeKey,
    ) -> impl Iterator<Item = &'structure MethodUse> + use<'structure> {
        self.methods.iter().filter(move |method| method.owner() == owner)
    }
}
