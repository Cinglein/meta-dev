mod finding;

use finding::Finding;
use syn::visit::Visit;
use syn::{Ident, ImplItemFn, ItemFn, ItemImpl, ItemMod, TraitItemFn};

use task::LineNumber;
use task::Name;
use task::Policy;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;

#[derive(Debug)]
pub struct LiteralNames;

impl LiteralNames {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "function names say what the function does",
            files
                .iter()
                .flat_map(|file| {
                    Self::vague_names(file.syntax(), policy)
                        .into_iter()
                        .map(move |name| {
                            Violation::new(
                                Site::Line(file.path().into(), LineNumber::from(name.span())),
                                Finding::VagueName {
                                    name: Name::from(&name),
                                },
                            )
                        })
                })
                .collect(),
        )
    }

    fn vague_names(file: &syn::File, policy: &Policy) -> Vec<Ident> {
        let mut names = VagueNames {
            policy,
            found: Vec::new(),
        };
        names.visit_file(file);
        names.found
    }
}

struct VagueNames<'scan> {
    policy: &'scan Policy,
    found: Vec<Ident>,
}

impl VagueNames<'_> {
    fn note(&mut self, name: &Ident) {
        self.found.extend(
            self.policy
                .is_vague(&Name::from(name))
                .then(|| name.clone()),
        );
    }
}

impl<'ast> Visit<'ast> for VagueNames<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.note(&function.sig.ident);
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.note(&function.sig.ident);
        syn::visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.note(&function.sig.ident);
        syn::visit::visit_trait_item_fn(self, function);
    }
}

#[cfg(test)]
mod tests {
    use super::LiteralNames;
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const SOURCE: &str = "
struct W;
impl W { fn of(x: u8) -> W { W } fn check(&self) {} fn new() -> W { W } fn least_significant_bit(&self) -> u8 { 0 } }
impl From<u8> for W { fn from(x: u8) -> W { W } } trait T { fn do_it(&self); fn go(&self); }
fn process() {} mod inner { fn handle() {} }
#[cfg(test)] mod tests { fn make() {} }
";
    const REPORTED: [&str; 5] = [
        ":3: fn of names nothing", ":3: fn check names nothing", ":4: fn go names nothing",
        ":5: fn process names nothing", ":5: fn handle names nothing",
    ];

    #[test]
    fn flags_prepositions_and_vague_verbs_but_not_new_trait_impls_or_tests() {
        let file = SourceFile::parse("w.rs", SOURCE.to_owned()).unwrap();
        let report = LiteralNames::report(core::slice::from_ref(&file), &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
