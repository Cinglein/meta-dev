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
