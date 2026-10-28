use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use syn::visit::Visit;
use syn::{
    Attribute, Expr, ExprCall, ExprMacro, ExprMethodCall, ExprPath, Ident, ImplItemFn, ItemImpl,
    ItemMod, Macro, Meta, PathArguments,
};

mod door;

use door::Door;

use super::finding::Finding;
use task::LineNumber;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;
use task::{Name, Whole};

pub struct TextSites {
    path: PathBuf,
    walk: Walk,
    within: Option<Door>,
    generics: Vec<Name>,
    doors: Vec<Door>,
    uses: Vec<Door>,
    found: BTreeMap<PathBuf, Vec<Ident>>,
}

impl TextSites {
    const INSPECTING: [&str; 8] = [
        "split_whitespace", "split_once", "trim", "trim_start", "trim_end", "chars",
        "char_indices", "split_at_checked",
    ];
    const PRODUCING: [&str; 2] = ["to_string", "push_str"];
    const TRAIT_OWNERS: [&str; 7] = [
        "FromStr", "TryFrom", "TryInto", "ToString", "Display", "str", "Self",
    ];

    pub fn violations(files: &[SourceFile], support: &[PathBuf]) -> Vec<Violation> {
        let empty = TextSites {
            path: PathBuf::new(),
            walk: Walk::Entered,
            within: None,
            generics: Vec::new(),
            doors: Vec::new(),
            uses: Vec::new(),
            found: BTreeMap::new(),
        };
        let sites = files.iter().fold(empty, |sites, file| {
            let mut scan = TextSites {
                path: file.path().to_owned(),
                walk: if support.contains(&file.path().to_owned()) {
                    Walk::Skipped
                } else {
                    Walk::Entered
                },
                ..sites
            };
            scan.visit_file(file.syntax());
            scan
        });
        sites
            .found
            .iter()
            .filter(|(path, _)| !sites.is_open(path))
            .flat_map(|(path, handlers)| {
                handlers.iter().map(|handler| {
                    Violation::new(
                        Site::Line(path.clone(), LineNumber::from(handler.span())),
                        Finding::TextOutsideDoor {
                            handler: Name::from(handler),
                        },
                    )
                })
            })
            .collect()
    }

    fn is_open(&self, path: &Path) -> bool {
        self.doors.iter().any(|door| {
            door.is_in(path)
                && self
                    .uses
                    .iter()
                    .any(|seen| door.is_reached_by(seen, &self.doors))
        })
    }

    fn noted(&mut self, callee: Option<&Ident>, target: Option<Name>) {
        let concrete = target.filter(|target| !self.generics.contains(target));
        let seen =
            callee.and_then(|callee| Door::of_callee(callee, concrete, self.within.as_ref()));
        self.uses.extend(seen);
    }

    fn sited(&mut self, handler: Option<Ident>) {
        let kept = handler.filter(|_| self.walk == Walk::Entered);
        self.found
            .entry(self.path.clone())
            .or_default()
            .extend(kept);
    }

    fn text_constructor(call: &ExprCall) -> Option<Ident> {
        let segments = match &*call.func {
            Expr::Path(path) => path.path.segments.iter().collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        match segments.as_slice() {
            [first, second] if first.ident == "String" && second.ident == "from" => {
                Some(second.ident.clone())
            }
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for TextSites {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.walk;
        self.walk = match (outer, SourceFile::walk(module)) {
            (Walk::Entered, Walk::Entered) => Walk::Entered,
            _ => Walk::Skipped,
        };
        syn::visit::visit_item_mod(self, module);
        self.walk = outer;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let site = Site::Line(self.path.clone(), LineNumber::from(item.impl_token.span));
        let door = Door::of_impl(item, site.clone());
        let outer = self.within.clone();
        let depth = self.generics.len();
        self.within = door.clone().or_else(|| outer.clone());
        self.doors
            .extend(door.filter(|_| self.walk == Walk::Entered));
        self.generics.extend(
            item.generics
                .type_params()
                .map(|parameter| Name::from(&parameter.ident)),
        );
        syn::visit::visit_item_impl(self, item);
        self.generics.truncate(depth);
        self.within = outer;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        let depth = self.generics.len();
        self.generics.extend(
            function
                .sig
                .generics
                .type_params()
                .map(|parameter| Name::from(&parameter.ident)),
        );
        syn::visit::visit_impl_item_fn(self, function);
        self.generics.truncate(depth);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let name = Name::from(&call.method);
        self.sited(
            (name.matches::<Whole>(&Self::INSPECTING) || name.matches::<Whole>(&Self::PRODUCING))
                .then(|| call.method.clone()),
        );
        self.noted(
            Some(&call.method),
            Door::first_type(call.turbofish.as_ref()),
        );
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, expr: &'ast ExprPath) {
        let last = expr.path.segments.last();
        let owner = expr
            .path
            .segments
            .iter()
            .rev()
            .nth(1)
            .map(|segment| Name::from(&segment.ident))
            .filter(|owner| !owner.matches::<Whole>(&Self::TRAIT_OWNERS));
        let generic = last.and_then(|segment| match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => Door::first_type(Some(arguments)),
            _ => None,
        });
        let qualified = expr
            .qself
            .as_ref()
            .and_then(|qself| Door::target_of(&qself.ty));
        self.noted(
            last.map(|segment| &segment.ident),
            generic.or(qualified).or(owner),
        );
        syn::visit::visit_expr_path(self, expr);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.sited(
            expr.mac
                .path
                .get_ident()
                .filter(|ident| *ident == "format")
                .cloned(),
        );
        syn::visit::visit_expr_macro(self, expr);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let seen = Door::in_tokens(invocation.tokens.clone(), self.within.as_ref());
        self.uses.extend(seen);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        let seen = match &attribute.meta {
            Meta::List(list) => Door::in_tokens(list.tokens.clone(), self.within.as_ref()),
            Meta::Path(_) | Meta::NameValue(_) => Vec::new(),
        };
        self.uses.extend(seen);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.sited(Self::text_constructor(call));
        syn::visit::visit_expr_call(self, call);
    }
}
