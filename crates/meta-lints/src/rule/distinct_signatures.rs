use std::collections::BTreeMap;

use quote::ToTokens;
use syn::visit::Visit;
use syn::{FnArg, ImplItem, ItemImpl, ItemMod, ReturnType, Signature, Visibility};

use crate::rule::report::Report;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct DistinctSignatures;

impl DistinctSignatures {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "two helpers on one type with the same signature hide a missing type",
            files
                .iter()
                .flat_map(|file| {
                    Self::shared(file.syntax()).into_iter().map(move |(shape, names)| {
                        Violation::new(
                            Site::File(file.path().to_owned()),
                            format!(
                                "{} share the signature {shape}; distinguish the types they take or return",
                                names.join(", ")
                            ),
                        )
                    })
                })
                .collect(),
        )
    }

    fn shared(file: &syn::File) -> BTreeMap<Shape, Vec<String>> {
        let mut index = SignatureIndex::default();
        index.visit_file(file);
        index.0.retain(|_, names| names.len() > 1);
        index.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Shape {
    self_type: String,
    generics: String,
    inputs: String,
    output: String,
}

impl Shape {
    fn describe(item: &ItemImpl, signature: &Signature) -> Shape {
        let inputs: Vec<String> = signature
            .inputs
            .iter()
            .map(|argument| match argument {
                FnArg::Receiver(receiver) => receiver.to_token_stream().to_string(),
                FnArg::Typed(typed) => typed.ty.to_token_stream().to_string(),
            })
            .map(|rendered| rendered.replace(' ', ""))
            .collect();
        Shape {
            self_type: item.self_ty.to_token_stream().to_string().replace(' ', ""),
            generics: signature.generics.to_token_stream().to_string().replace(' ', ""),
            inputs: inputs.join(", "),
            output: match &signature.output {
                ReturnType::Default => String::new(),
                ReturnType::Type(_, ty) => format!(" -> {}", ty.to_token_stream()).replace(' ', ""),
            },
        }
    }
}

impl std::fmt::Display for Shape {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}::{}({}){}", self.self_type, self.generics, self.inputs, self.output.replace("->", " -> "))
    }
}

#[derive(Default)]
struct SignatureIndex(BTreeMap<Shape, Vec<String>>);

impl<'ast> Visit<'ast> for SignatureIndex {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        item.items
            .iter()
            .filter(|_| item.trait_.is_none())
            .filter_map(|member| match member {
                ImplItem::Fn(function) if !matches!(function.vis, Visibility::Public(_)) => {
                    Some((Shape::describe(item, &function.sig), function.sig.ident.to_string()))
                }
                _ => None,
            })
            .for_each(|(shape, name)| self.0.entry(shape).or_default().push(name));
    }
}

#[cfg(test)]
mod tests {
    use super::DistinctSignatures;

    const SOURCE: &str = "
struct W;
impl W {
    fn alpha(self) -> u8 { 0 }
    fn beta(self) -> u8 { 0 }
    pub fn gamma(self) -> u8 { 0 }
    fn delta(&self) -> u8 { 0 }
    pub(crate) fn epsilon(self, other: u8) -> u8 { other }
}
";
    const SHARED: [&str; 2] = ["alpha", "beta"];

    #[test]
    fn groups_non_pub_fns_of_one_type_by_receiver_parameters_and_return() {
        let parsed = syn::parse_file(SOURCE).expect("valid rust");
        let shared = DistinctSignatures::shared(&parsed);
        assert_eq!(shared.len(), 1);
        assert_eq!(shared.into_values().next(), Some(SHARED.map(String::from).to_vec()));
    }
}
