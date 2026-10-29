use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{ExprMethodCall, ExprPath, GenericArgument, Item, Type};

use task::Name;

use crate::effect::Effect;
use crate::item_graph::scope::Scope;

#[derive(Debug)]
pub struct EffectPaths<'scope> {
    scope: &'scope Scope,
    found: BTreeSet<Effect>,
}

impl<'scope> EffectPaths<'scope> {
    const PARSE: &str = "parse";
    const FROM_STR: &str = "from_str";

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
    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        self.found
            .extend(Effect::performed_by(&self.scope.resolved(&expression.path)));
        visit::visit_expr_path(self, expression);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let scope = self.scope;
        let parsed = call
            .turbofish
            .iter()
            .filter(|_| call.method == Self::PARSE)
            .flat_map(|turbofish| turbofish.args.iter())
            .filter_map(|argument| match argument {
                GenericArgument::Type(Type::Path(target)) => Some(
                    scope
                        .resolved(&target.path)
                        .extended(Name::from(Self::FROM_STR)),
                ),
                _ => None,
            })
            .filter_map(|function| Effect::performed_by(&function));
        self.found.extend(parsed);
        visit::visit_expr_method_call(self, call);
    }
}
