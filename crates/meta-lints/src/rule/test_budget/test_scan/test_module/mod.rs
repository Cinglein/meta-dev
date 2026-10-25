mod large_literal;
mod macro_arguments;
mod scope;
mod test_counts;
mod test_over;

use large_literal::LargeLiteral;
use macro_arguments::MacroArguments;
use scope::Scope;
use test_counts::TestCounts;
use test_over::TestOver;

use std::mem;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemFn, ItemMod, LitInt, Macro};

use super::super::TestBudget;
use super::super::amount::Amount;
use super::super::measurement::Measurement;
use crate::rule::name::Name;
use crate::site::Site;
use crate::violation::Violation;

pub(super) struct TestModule<'scan> {
    path: &'scan str,
    scope: Scope,
    budget: TestBudget,
}

impl<'scan> TestModule<'scan> {
    pub(super) fn budget(path: &'scan str, module: &ItemMod) -> TestBudget {
        let mut scan = TestModule {
            path,
            scope: Scope::OutsideTest,
            budget: TestBudget { test_lines: Amount::lines_spanned(module), ..TestBudget::default() },
        };
        scan.visit_item_mod(module);
        scan.budget
    }

    pub(super) fn file_budget(path: &'scan str, file: &syn::File) -> TestBudget {
        let mut scan = TestModule { path, scope: Scope::OutsideTest, budget: TestBudget::default() };
        scan.visit_file(file);
        scan.budget
    }

    fn test_budget(&self, function: &ItemFn) -> TestBudget {
        let name = &function.sig.ident;
        let site = Site::spanned(self.path, name.span());
        TestBudget {
            tests: Amount::ONE,
            violations: TestCounts::tally(&function.block)
                .measurements()
                .into_iter()
                .filter(Measurement::exceeded)
                .map(|measurement| Violation::new(site.clone(), TestOver::new(Name::from(name), measurement)))
                .collect(),
            ..TestBudget::default()
        }
    }

    fn is_test(function: &ItemFn) -> bool {
        function.attrs.iter().any(|attribute| attribute.path().is_ident("test"))
    }
}

impl<'ast> Visit<'ast> for TestModule<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let counted = if Self::is_test(function) { self.test_budget(function) } else { TestBudget::default() };
        self.budget = mem::take(&mut self.budget) + counted;
        self.scope = if Self::is_test(function) { Scope::InsideTest } else { self.scope };
        syn::visit::visit_item_fn(self, function);
        self.scope = Scope::OutsideTest;
    }

    fn visit_lit_int(&mut self, integer: &'ast LitInt) {
        let large =
            integer.base10_parse::<Amount>().ok().filter(|value| *value > TestBudget::MAX_INTEGER_LITERAL).map(|_| {
                Violation::new(
                    Site::spanned(self.path, integer.span()),
                    LargeLiteral::new(integer.clone(), TestBudget::MAX_INTEGER_LITERAL),
                )
            });
        self.budget.violations.extend(large);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let stray = (self.scope == Scope::OutsideTest && TestCounts::is_assertion(invocation)).then(|| {
            Violation::new(
                Site::spanned(self.path, invocation.span()),
                "assertion outside a test; helpers return values and tests assert them",
            )
        });
        self.budget.violations.extend(stray);
        MacroArguments::parse(invocation).visit_with(self);
    }
}
