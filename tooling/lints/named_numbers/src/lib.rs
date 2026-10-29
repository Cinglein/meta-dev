use std::path::PathBuf;

use syn::visit::Visit;
use syn::{
    ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, TraitItemConst,
};

use syn_rules::MacroExpressions;
use task::LineNumber;
use task::Policy;
use task::Report;
use task::Site;
use task::SourceFile;
use task::SupportModules;
use task::Violation;
use task::Walk;

#[derive(Debug)]
pub struct NamedNumbers;

impl NamedNumbers {
    const UNITS: &[&str] = &["0", "1", "0.0", "1.0"];

    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            "a number has a name: outside tests, a literal other than 0 and 1 sits in a const item",
            files
                .iter()
                .filter(|file| {
                    !policy.bit_boundary().covers(file.path()) && !support.contains(&file.path().to_owned())
                })
                .flat_map(|file| {
                    let mut numbers = Numbers::default();
                    numbers.visit_file(file.syntax());
                    numbers.0.into_iter().map(move |line| {
                        Violation::new(
                            Site::Line(file.path().into(), line),
                            "numeric literal outside a const item; name it as a const so it can be found and tuned",
                        )
                    })
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn is_unit(digits: &str) -> bool {
        Self::UNITS.contains(&digits.trim_start_matches('-'))
    }

    fn unnamed(digits: &str, line: LineNumber) -> Option<LineNumber> {
        (!Self::is_unit(digits)).then_some(line)
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

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::outside_tests(&item.attrs) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_trait_item_const(&mut self, _: &'ast TraitItemConst) {}

    fn visit_expr_const(&mut self, _: &'ast ExprConst) {}

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        MacroExpressions::visit_parsed(invocation, self);
    }

    fn visit_lit_int(&mut self, literal: &'ast LitInt) {
        self.0.extend(NamedNumbers::unnamed(
            literal.base10_digits(),
            LineNumber::from(literal.span()),
        ));
    }

    fn visit_lit_float(&mut self, literal: &'ast LitFloat) {
        self.0.extend(NamedNumbers::unnamed(
            literal.base10_digits(),
            LineNumber::from(literal.span()),
        ));
    }
}
