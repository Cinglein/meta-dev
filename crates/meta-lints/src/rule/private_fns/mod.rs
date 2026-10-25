mod crowded_type;
mod private_fn_limit;
mod self_type;

use std::collections::BTreeMap;

use crowded_type::CrowdedType;
use private_fn_limit::PrivateFnLimit;
use self_type::SelfType;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, Visibility};

use crate::rule::name::Name;
use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct PrivateFns;

impl PrivateFns {
    const MAX_PRIVATE_FNS_PER_TYPE: PrivateFnLimit = PrivateFnLimit::new(4);

    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "too many private fns; a type's logic belongs in its interface or in more types",
            files
                .iter()
                .flat_map(|file| {
                    Self::private_fns(file.syntax())
                        .into_iter()
                        .filter(|(_, names)| Self::MAX_PRIVATE_FNS_PER_TYPE.exceeded_by(names))
                        .map(move |(self_type, names)| {
                            Violation::new(
                                Site::File(file.path().to_owned()),
                                CrowdedType::new(self_type, names, Self::MAX_PRIVATE_FNS_PER_TYPE),
                            )
                        })
                })
                .collect(),
        )
    }

    fn private_fns(file: &syn::File) -> BTreeMap<SelfType, Vec<Name>> {
        let mut counter = PrivateFnCounter::default();
        counter.visit_file(file);
        counter.0
    }
}

#[derive(Default)]
struct PrivateFnCounter(BTreeMap<SelfType, Vec<Name>>);

impl<'ast> Visit<'ast> for PrivateFnCounter {
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let names = item.items.iter().filter_map(|item| match item {
            ImplItem::Fn(function) if matches!(function.vis, Visibility::Inherited) => {
                Some(Name::from(&function.sig.ident))
            }
            _ => None,
        });
        if item.trait_.is_none() {
            self.0.entry(SelfType::from(&*item.self_ty)).or_default().extend(names);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PrivateFns;

    const SOURCE: &str = "
struct Wide;
impl Wide { fn a() {} fn b() {} pub fn c() {} }
impl Wide { fn d() {} fn e() {} pub(crate) fn f() {} }
impl Iterator for Wide { type Item = u8; fn next(&mut self) -> Option<u8> { None } }
";
    const HIDDEN: [&str; 4] = ["a", "b", "d", "e"];

    #[test]
    fn counts_private_inherent_fns_across_impl_blocks_but_not_trait_impls() {
        let parsed = syn::parse_file(SOURCE).expect("valid rust");
        let counted = PrivateFns::private_fns(&parsed);
        let wide: Vec<String> = counted
            .iter()
            .filter(|(self_type, _)| self_type.to_string() == "Wide")
            .flat_map(|(_, names)| names.iter().map(ToString::to_string))
            .collect();
        assert_eq!(wide, HIDDEN);
    }
}
