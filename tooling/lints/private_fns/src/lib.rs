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

#[cfg(test)]
mod tests {
    use super::PrivateFns;
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const SOURCE: &str = "
struct Wide;
impl Wide { fn a() {} fn b() {} pub fn c() {} }
impl Wide { fn d() {} fn e() {} fn g() {} pub(crate) fn f() {} }
impl Iterator for Wide { type Item = u8; fn next(&mut self) -> Option<u8> { None } }
struct Narrow;
impl Narrow { fn a() {} fn b() {} fn c() {} fn d() {} }
";
    const REPORTED: [&str; 1] = ["Wide has 5 private fns, at most 4 allowed: a, b, d, e, g"];

    #[test]
    fn counts_private_inherent_fns_across_impl_blocks_but_not_trait_impls() {
        let file = SourceFile::parse("wide.rs", SOURCE.to_owned()).unwrap();
        let report = PrivateFns::report(core::slice::from_ref(&file), &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
