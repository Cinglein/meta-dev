mod text_outside_door;

use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMacro, ExprMethodCall, Ident, ImplItemFn, ItemImpl, ItemMod};

use crate::region::Region;
use crate::rule::name::Name;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;
use text_outside_door::TextOutsideDoor;

#[derive(Default)]
pub struct TextSites {
    doors: Vec<Ident>,
    found: Vec<Ident>,
}

impl TextSites {
    const DOOR_TRAITS: [&str; 3] = ["FromStr", "TryFrom", "Display"];
    const INSPECTING: [&str; 13] = [
        "split_whitespace",
        "split_once",
        "strip_prefix",
        "strip_suffix",
        "starts_with",
        "ends_with",
        "trim",
        "trim_start",
        "trim_end",
        "chars",
        "bytes",
        "char_indices",
        "split_at_checked",
    ];
    const PRODUCING: [&str; 2] = ["to_string", "push_str"];

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut sites = TextSites::default();
        sites.visit_file(file.syntax());
        if file.region() != Region::Product || !sites.doors.is_empty() {
            Vec::new()
        } else {
            sites
                .found
                .into_iter()
                .map(|ident| {
                    Violation::new(Site::spanned(file.path(), ident.span()), TextOutsideDoor::new(Name::from(&ident)))
                })
                .collect()
        }
    }

    fn string_from(call: &ExprCall) -> Option<Ident> {
        match &*call.func {
            Expr::Path(path) => {
                match path.path.segments.iter().map(|segment| &segment.ident).collect::<Vec<&Ident>>().as_slice() {
                    [first, second] if *first == "String" && *second == "from" => Some((*second).clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn door_trait(item: &ItemImpl) -> Option<Ident> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| segment.ident.clone())
            .filter(|ident| Self::DOOR_TRAITS.iter().any(|door| ident == door))
    }
}

impl<'ast> Visit<'ast> for TextSites {
    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        if !SourceFile::is_test_item(&function.attrs) {
            syn::visit::visit_impl_item_fn(self, function);
        }
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.doors.extend(Self::door_trait(item));
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.found.extend(
            Some(&call.method)
                .filter(|method| Self::INSPECTING.iter().chain(&Self::PRODUCING).any(|name| *method == name))
                .cloned(),
        );
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.found.extend(expr.mac.path.get_ident().filter(|_| expr.mac.path.is_ident("format")).cloned());
        syn::visit::visit_expr_macro(self, expr);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.found.extend(Self::string_from(call));
        syn::visit::visit_expr_call(self, call);
    }
}
