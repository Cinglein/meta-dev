use std::path::PathBuf;

use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{
    Expr, ExprConst, ImplItemConst, ItemConst, ItemImpl, ItemMod, LitFloat, LitInt, Macro, Token,
    TraitItemConst,
};

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
        invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .into_iter()
            .flatten()
            .for_each(|argument| self.visit_expr(&argument));
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

#[cfg(test)]
mod tests {
    use super::NamedNumbers;
    use settings::EmbeddedDefaults;
    use task::Policy;
    use task::SourceFile;
    const BIT_BOUNDARY: &str = "bit_boundary";
    const BOUNDARY: &str = "crates/board/src/direction.rs";
    const SOURCE: &str = "
pub struct Clock;
impl Clock { const HALF: u64 = 500; pub fn halved(ticks: u64) -> u64 { ticks / 2 + 1 } }
impl Clock { pub fn scaled(ticks: f64) -> f64 { ticks * 0.5 } pub fn named(ticks: u64) -> u64 { ticks / Self::HALF } pub fn unit(ticks: u64) -> u64 { ticks - 1 } }
#[cfg(any(test, feature = \"proptest\"))] impl Clock { pub fn range() -> u64 { 60 } }
#[cfg(feature = \"metal\")] impl Clock { pub fn lanes() -> u64 { 8 } }
pub trait Rate { const BASE: f64 = 2.5; fn beat(&self) -> u64 { 3 } } const LIMIT: u64 = 40;
pub fn capped(ticks: u64) -> u64 { let edge = const { 7 }; assert!(ticks > 9); ticks.min(LIMIT) }
mod inner { pub fn depth() -> u8 { 4 } }
#[cfg(test)] mod tests { fn depth() -> u8 { 4 } }
";
    const FILES: [(&str, &str); 2] = [
        ("crates/a/src/clock.rs", SOURCE),
        (BOUNDARY, "pub fn step() -> i8 { 8 }"),
    ];
    const REPORTED: [&str; 6] = [
        "clock.rs:3:", "clock.rs:4:", "clock.rs:6:", "clock.rs:7:", "clock.rs:8:", "clock.rs:9:",
    ];

    #[test]
    fn flags_literals_outside_consts_but_not_units_test_only_impls_or_bit_boundary_files() {
        let mut table: toml::Table = toml::from_str(Policy::DEFAULTS).unwrap();
        table.insert(BIT_BOUNDARY.to_owned(), toml::Value::from(vec![BOUNDARY]));
        let policy: Policy = table.try_into().unwrap();
        let files = FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let report = NamedNumbers::report(&files, &policy);
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
