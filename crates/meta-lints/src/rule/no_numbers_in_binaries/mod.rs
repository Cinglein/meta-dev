use syn::visit::Visit;
use syn::{ItemMod, LitFloat, LitInt};

use crate::line_number::LineNumber;
use crate::rule::binary_files::BinaryFiles;
use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct NoNumbersInBinaries;

impl NoNumbersInBinaries {
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
                            Site::Line(file.path().to_owned(), line),
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
        Some(module)
            .filter(|module| !SourceFile::is_test_item(&module.attrs))
            .into_iter()
            .for_each(|module| syn::visit::visit_item_mod(self, module));
    }

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.0.push(LineNumber::starting(literal.span()));
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.push(LineNumber::starting(literal.span()));
    }
}

#[cfg(test)]
mod tests {
    use super::NoNumbersInBinaries;
    use crate::source_file::SourceFile;

    const MAIN: &str = "mod worker; fn main() {}";
    const WORKER: &str = "pub struct W; impl W { pub fn size() -> usize { 42 } }";
    const LIB: &str = "pub struct L; impl L { pub fn size() -> usize { 42 } }";
    const FILES: [(&str, &str); 3] =
        [("crates/a/src/main.rs", MAIN), ("crates/a/src/worker.rs", WORKER), ("crates/a/src/lib.rs", LIB)];

    #[test]
    fn flags_a_literal_in_a_module_reached_from_main_but_not_in_the_library() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path.to_owned(), text.to_owned()).expect("valid rust"));
        let report = NoNumbersInBinaries::report(&files).to_string();
        assert!(report.contains("worker.rs") && !report.contains("lib.rs"), "{report}");
    }
}
