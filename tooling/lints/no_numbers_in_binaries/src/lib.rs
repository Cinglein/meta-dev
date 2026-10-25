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

#[cfg(test)]
mod tests {
    use super::NoNumbersInBinaries;
    use task::SourceFile;
    const WORKER: &str = "pub struct W; impl W { pub fn size() -> usize { 42 } pub fn rate() -> f64 { 2.5 } }\nmod inner { pub fn depth() -> u8 { 7 } }\n#[cfg(test)]\nmod tests { fn depth() -> u8 { 7 } }";
    const FILES: [(&str, &str); 3] = [
        ("crates/a/src/main.rs", "mod worker; fn main() {}"),
        ("crates/a/src/worker.rs", WORKER),
        (
            "crates/a/src/lib.rs",
            "pub struct L; impl L { pub fn size() -> usize { 42 } }",
        ),
    ];
    const REPORTED: [&str; 3] = ["worker.rs:1:", "worker.rs:1:", "worker.rs:2:"];

    #[test]
    fn flags_literals_in_a_module_reached_from_main_but_not_in_the_library_or_tests() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let report = NoNumbersInBinaries::report(&files);
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
