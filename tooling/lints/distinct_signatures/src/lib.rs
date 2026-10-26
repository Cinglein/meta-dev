use std::collections::BTreeMap;

use quote::ToTokens;
use syn::visit::Visit;
use syn::{FnArg, ImplItem, ImplItemFn, ItemImpl, ItemMod, ReturnType, Signature};

use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;

#[derive(Debug)]
pub struct DistinctSignatures;

impl DistinctSignatures {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "two functions on one type with the same signature hide a missing type or type parameter",
            files
                .iter()
                .flat_map(|file| {
                    Self::shared(file.syntax())
                        .into_iter()
                        .map(move |(shape, names)| {
                            Violation::new(
                                Site::File(file.path().into()),
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
            generics: signature
                .generics
                .to_token_stream()
                .to_string()
                .replace(' ', ""),
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
        write!(
            formatter,
            "{}::{}({}){}",
            self.self_type,
            self.generics,
            self.inputs,
            self.output.replace("->", " -> ")
        )
    }
}

#[derive(Default)]
struct SignatureIndex(BTreeMap<Shape, Vec<String>>);

impl SignatureIndex {
    fn functions(item: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> {
        item.items.iter().filter_map(|member| match member {
            ImplItem::Fn(function) => Some(function),
            _ => None,
        })
    }
}

impl<'ast> Visit<'ast> for SignatureIndex {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => Self::functions(item).for_each(|function| {
                self.0
                    .entry(Shape::describe(item, &function.sig))
                    .or_default()
                    .push(function.sig.ident.to_string());
            }),
            Walk::Skipped => {}
        }
    }
}
