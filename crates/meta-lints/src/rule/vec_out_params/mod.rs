mod vec_out_fn;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{FnArg, Signature, Type};

use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;
use vec_out_fn::VecOutFn;

pub struct VecOutParams<'file> {
    path: &'file str,
    violations: Vec<Violation>,
}

impl VecOutParams<'_> {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules",
            files
                .iter()
                .flat_map(|file| {
                    let mut scan = VecOutParams { path: file.path(), violations: Vec::new() };
                    scan.visit_file(file.syntax());
                    scan.violations
                })
                .collect(),
        )
    }

    fn is_vec_out(ty: &Type) -> bool {
        match ty {
            Type::Reference(reference) => {
                reference.mutability.is_some()
                    && matches!(
                        reference.elem.as_ref(),
                        Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Vec")
                    )
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for VecOutParams<'_> {
    fn visit_signature(&mut self, signature: &'ast Signature) {
        let path = self.path;
        self.violations.extend(
            signature
                .inputs
                .iter()
                .filter_map(|input| match input {
                    FnArg::Typed(typed) => Some(typed),
                    FnArg::Receiver(_) => None,
                })
                .filter(|typed| Self::is_vec_out(&typed.ty))
                .map(|typed| Violation::new(Site::spanned(path, typed.span()), VecOutFn::new(&signature.ident))),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::VecOutParams;
    use crate::source_file::SourceFile;

    const SOURCE: &str = "
impl Placer {
    fn place(&self, taken: &mut Vec<u8>) {}
    fn read(&self, taken: &Vec<u8>) {}
    fn own(&mut self, taken: Vec<u8>) {}
}
";

    #[test]
    fn flags_only_mutable_vec_parameters() {
        let file = SourceFile::parse("placer.rs".to_owned(), SOURCE.to_owned()).expect("valid rust");
        let report = VecOutParams::report(&[file]).to_string();
        assert_eq!(report.lines().skip(1).collect::<Vec<_>>(), ["placer.rs:3: fn place"]);
    }
}
