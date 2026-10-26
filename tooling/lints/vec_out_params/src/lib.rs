mod finding;

use finding::Finding;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{FnArg, Signature, Type};

use task::{LineNumber, Name, Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct VecOutParams<'file> {
    file: &'file SourceFile,
    violations: Vec<Violation>,
}

impl VecOutParams<'_> {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules",
            files
                .iter()
                .flat_map(|file| {
                    let mut scan = VecOutParams {
                        file,
                        violations: Vec::new(),
                    };
                    scan.visit_file(file.syntax());
                    scan.violations
                })
                .collect(),
        )
    }

    fn is_vec_out(parameter: &Type) -> bool {
        match parameter {
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
        let file = self.file;
        self.violations.extend(
            signature
                .inputs
                .iter()
                .filter_map(|input| match input {
                    FnArg::Typed(typed) => Some(typed),
                    FnArg::Receiver(_) => None,
                })
                .filter(|typed| Self::is_vec_out(&typed.ty))
                .map(|typed| {
                    Violation::new(
                        Site::Line(file.path().into(), LineNumber::from(typed.span())),
                        Finding::VecOut {
                            name: Name::from(&signature.ident),
                        },
                    )
                }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::VecOutParams;
    use task::SourceFile;
    const SOURCE: &str = "
impl Placer {
    fn place(&self, taken: &mut Vec<u8>) {}
    fn read(&self, taken: &Vec<u8>) {}
    fn own(&mut self, taken: Vec<u8>) {}
}
";
    const REPORTED: [&str; 1] = [":3: fn place"];

    #[test]
    fn flags_only_mutable_vec_parameters() {
        let file = SourceFile::parse("placer.rs", SOURCE.to_owned()).unwrap();
        let report = VecOutParams::report(core::slice::from_ref(&file));
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
