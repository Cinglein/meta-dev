use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{Item, Path};

use crate::effect::Effect;
use crate::item_graph::scope::Scope;

#[derive(Debug)]
pub struct EffectPaths<'scope> {
    scope: &'scope Scope,
    found: BTreeSet<Effect>,
}

impl<'scope> EffectPaths<'scope> {
    pub fn in_item(scope: &'scope Scope, item: &Item) -> BTreeSet<Effect> {
        let mut paths = EffectPaths {
            scope,
            found: BTreeSet::new(),
        };
        paths.visit_item(item);
        paths.found
    }
}

impl<'ast> Visit<'ast> for EffectPaths<'_> {
    fn visit_path(&mut self, path: &'ast Path) {
        self.found
            .extend(Effect::touched_by(&self.scope.resolved(path)));
        visit::visit_path(self, path);
    }
}
