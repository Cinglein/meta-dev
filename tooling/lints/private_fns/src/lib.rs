mod finding;

use std::collections::BTreeMap;

use finding::Finding;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, Visibility};

use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::TypeName;
use task::Violation;
use task::Walk;
use task::{Limit, Policy};

#[derive(Debug)]
pub struct PrivateFns;

impl PrivateFns {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "too many private fns; a type's logic belongs in its interface or in more types",
            files
                .iter()
                .flat_map(|file| {
                    Self::private_fns(file.syntax())
                        .into_iter()
                        .filter(|(_, names)| {
                            !policy.limit(Limit::PrivateFnsPerType).allows(names.len())
                        })
                        .map(move |(type_name, names)| {
                            Violation::new(
                                Site::File(file.path().into()),
                                Finding::TooManyPrivateFns {
                                    type_name,
                                    names,
                                    most: policy.limit(Limit::PrivateFnsPerType),
                                },
                            )
                        })
                })
                .collect(),
        )
    }

    fn private_fns(file: &syn::File) -> BTreeMap<TypeName, Vec<Name>> {
        let mut counter = PrivateFnCounter::default();
        counter.visit_file(file);
        counter.0
    }
}

#[derive(Default)]
struct PrivateFnCounter(BTreeMap<TypeName, Vec<Name>>);

impl PrivateFnCounter {
    fn private_names(item: &ItemImpl) -> impl Iterator<Item = Name> {
        item.items.iter().filter_map(|member| match member {
            ImplItem::Fn(function) if matches!(function.vis, Visibility::Inherited) => {
                Some(Name::from(&function.sig.ident))
            }
            _ => None,
        })
    }
}

impl<'ast> Visit<'ast> for PrivateFnCounter {
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => self
                .0
                .entry(TypeName::from_type(&item.self_ty))
                .or_default()
                .extend(Self::private_names(item)),
            Walk::Skipped => {}
        }
    }
}
