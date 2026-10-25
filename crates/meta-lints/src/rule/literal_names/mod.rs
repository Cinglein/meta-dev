mod vague_name;

use syn::visit::Visit;
use syn::{ImplItemFn, ItemFn, ItemImpl, ItemMod, TraitItemFn};
use vague_name::VagueName;

use crate::rule::report::Report;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct LiteralNames;

impl LiteralNames {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "function names say what the function does",
            files
                .iter()
                .flat_map(|file| {
                    Self::vague_names(file.syntax())
                        .into_iter()
                        .map(move |name| Violation::new(Site::spanned(file.path(), name.name().span()), name))
                })
                .collect(),
        )
    }

    fn vague_names(file: &syn::File) -> Vec<VagueName> {
        let mut names = VagueNames::default();
        names.visit_file(file);
        names.0
    }
}

#[derive(Default)]
struct VagueNames(Vec<VagueName>);

impl<'ast> Visit<'ast> for VagueNames {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if item.trait_.is_none() {
            syn::visit::visit_item_impl(self, item);
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.0.extend(VagueName::try_from(&function.sig.ident));
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.0.extend(VagueName::try_from(&function.sig.ident));
        syn::visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.0.extend(VagueName::try_from(&function.sig.ident));
        syn::visit::visit_trait_item_fn(self, function);
    }
}

#[cfg(test)]
mod tests {
    use super::LiteralNames;

    const SOURCE: &str = "
struct W;
impl W {
    fn of(x: u8) -> W { W }
    fn check(&self) {}
    fn new() -> W { W }
    fn least_significant_bit(&self) -> u8 { 0 }
}
impl From<u8> for W { fn from(x: u8) -> W { W } }
trait T { fn do_it(&self); fn go(&self); }
";
    const FLAGGED: [&str; 3] = ["of", "check", "go"];

    #[test]
    fn flags_prepositions_and_vague_verbs_but_not_new_or_trait_impls() {
        let parsed = syn::parse_file(SOURCE).expect("valid rust");
        let names: Vec<String> =
            LiteralNames::vague_names(&parsed).iter().map(|vague| vague.name().to_string()).collect();
        assert_eq!(names, FLAGGED);
    }
}
