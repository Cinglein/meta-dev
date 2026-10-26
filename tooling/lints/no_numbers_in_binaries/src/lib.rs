use syn::visit::Visit;
use syn::{ItemMod, LitFloat, LitInt};

use named_numbers::NamedNumbers;
use task::BinaryFiles;
use task::LineNumber;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;

#[derive(Debug)]
pub struct NoNumbersInBinaries;

impl NoNumbersInBinaries {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let binaries = BinaryFiles::collect(files);
        Report::new(
            "a binary is wiring: a number it needs is a setting in its config file",
            files
                .iter()
                .filter(|file| binaries.contains(file.path()))
                .flat_map(|file| {
                    let mut numbers = Numbers::default();
                    numbers.visit_file(file.syntax());
                    numbers.0.into_iter().map(move |line| {
                        Violation::new(
                            Site::Line(file.path().into(), line),
                            "numeric literal in a binary; read it from the settings file or its embedded defaults",
                        )
                    })
                })
                .collect(),
        )
    }
}

#[derive(Default)]
struct Numbers(Vec<LineNumber>);

impl<'ast> Visit<'ast> for Numbers {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.0.extend(
            (!NamedNumbers::is_unit(literal.base10_digits()))
                .then(|| LineNumber::from(literal.span())),
        );
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.push(LineNumber::from(literal.span()));
    }
}
