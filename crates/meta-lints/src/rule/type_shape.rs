use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{FnArg, GenericArgument, ItemImpl, ItemMod, ItemStruct, PathArguments, Signature, Type, TypePath, TypeTuple};

use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct TypeShape;

impl TypeShape {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "types carry their meaning: no bool fields, no tuples in signatures, no String errors",
            files.iter().flat_map(Self::violations).collect(),
        )
    }

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut shapes = TypeShapes { path: file.path(), violations: Vec::new() };
        shapes.visit_file(file.syntax());
        shapes.violations
    }

    fn findings(ty: &Type) -> Vec<&'static str> {
        let mut findings = TypeFindings::default();
        findings.visit_type(ty);
        findings.0
    }

    fn is_string_result(path: &TypePath) -> bool {
        path.path
            .segments
            .last()
            .filter(|segment| segment.ident == "Result")
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.iter().nth(1),
                _ => None,
            })
            .is_some_and(
                |error| matches!(error, GenericArgument::Type(Type::Path(error)) if error.path.is_ident("String")),
            )
    }
}

struct TypeShapes<'scan> {
    path: &'scan str,
    violations: Vec<Violation>,
}

impl TypeShapes<'_> {
    fn report(&mut self, span: Span, message: &str) {
        self.violations.push(Violation::new(Site::spanned(self.path, span), message));
    }

    fn check_type(&mut self, ty: &Type) {
        TypeShape::findings(ty).into_iter().for_each(|finding| self.report(ty.span(), finding));
    }
}

impl<'ast> Visit<'ast> for TypeShapes<'_> {
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

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        item.fields.iter().for_each(|field| {
            if matches!(&field.ty, Type::Path(path) if path.path.is_ident("bool")) {
                self.report(
                    field.span(),
                    "bool field; a stored flag is a stored mode, make it an enum or a type parameter",
                );
            }
            self.check_type(&field.ty);
        });
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        signature
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                FnArg::Typed(typed) => Some(&*typed.ty),
                FnArg::Receiver(_) => None,
            })
            .for_each(|ty| self.check_type(ty));
        match &signature.output {
            syn::ReturnType::Type(_, ty) => self.check_type(ty),
            syn::ReturnType::Default => {}
        }
    }
}

#[derive(Default)]
struct TypeFindings(Vec<&'static str>);

impl<'ast> Visit<'ast> for TypeFindings {
    fn visit_type_tuple(&mut self, tuple: &'ast TypeTuple) {
        if !tuple.elems.is_empty() {
            self.0.push("tuple in a signature or field; values that travel together are a struct");
        }
        syn::visit::visit_type_tuple(self, tuple);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        if TypeShape::is_string_result(path) {
            self.0.push("Result with a String error; errors are an enum with thiserror");
        }
        syn::visit::visit_type_path(self, path);
    }
}

#[cfg(test)]
mod tests {
    use super::TypeShape;
    use crate::source_file::SourceFile;

    const SOURCE: &str = "
struct Flags { ready: bool, pair: (u8, u8) }
impl Flags {
    fn split(&self) -> (u8, u8) { self.pair }
    fn load(&self) -> Result<u8, String> { Ok(0) }
    fn unit(&self) -> () {}
}
impl Iterator for Flags { type Item = u8; fn size_hint(&self) -> (usize, Option<usize>) { (0, None) } }
";
    const REPORTED: [&str; 4] =
        [":2: bool field", ":2: tuple in a signature", ":4: tuple in a signature", ":5: Result with a String error"];

    #[test]
    fn flags_bool_fields_tuples_and_string_errors_outside_trait_impls() {
        let file = SourceFile::parse("flags.rs".to_owned(), SOURCE.to_owned()).expect("valid rust");
        let report = TypeShape::report(core::slice::from_ref(&file)).to_string();
        assert!(REPORTED.iter().all(|line| report.contains(line)) && !report.contains(":8:"), "{report}");
    }
}
