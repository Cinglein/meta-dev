mod macro_arguments;
mod scope;
mod test_counts;

use macro_arguments::MacroArguments;
use scope::Scope;
use test_counts::TestCounts;

use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{ItemFn, ItemMod, LitInt, Macro};

use super::super::TestBudget;
use super::super::line_count::LineCount;
use super::super::test_count::TestCount;
use super::finding::Finding;
use task::LineNumber;
use task::Name;
use task::Site;
use task::Violation;
use task::{Limit, Policy};

pub(super) struct TestModule<'scan> {
    path: &'scan Path,
    policy: &'scan Policy,
    scope: Scope,
    budget: TestBudget,
}

impl<'scan> TestModule<'scan> {
    pub(super) fn budget(path: &'scan Path, module: &ItemMod, policy: &'scan Policy) -> TestBudget {
        let mut scan = TestModule {
            path,
            policy,
            scope: Scope::OutsideTest,
            budget: TestBudget {
                test_lines: Self::line_count(module.span()),
                ..TestBudget::default()
            },
        };
        scan.visit_item_mod(module);
        scan.budget
    }

    fn test_violations(&self, function: &ItemFn) -> Vec<Violation> {
        let name = &function.sig.ident;
        let site = Site::Line(self.path.into(), LineNumber::from(name.span()));
        TestCounts::tally(&function.block)
            .overruns(self.policy)
            .into_iter()
            .map(|overrun| {
                Violation::new(
                    site.clone(),
                    Finding::TestOverrun {
                        name: Name::from(name),
                        overrun,
                    },
                )
            })
            .collect()
    }

    fn line_count(span: Span) -> LineCount {
        LineCount::from(span.end().line - span.start().line + 1)
    }

    fn scope_of(function: &ItemFn) -> Scope {
        if function
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("test"))
        {
            Scope::InsideTest
        } else {
            Scope::OutsideTest
        }
    }
}

impl<'ast> Visit<'ast> for TestModule<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.scope = Self::scope_of(function);
        let violations = match self.scope {
            Scope::InsideTest => self.test_violations(function),
            Scope::OutsideTest => Vec::new(),
        };
        self.budget.tests += TestCount::from(usize::from(self.scope == Scope::InsideTest));
        self.budget.violations.extend(violations);
        syn::visit::visit_item_fn(self, function);
        self.scope = Scope::OutsideTest;
    }

    fn visit_lit_int(&mut self, integer: &'ast LitInt) {
        let largest = self.policy.limit(Limit::LargestInteger);
        let finding = integer
            .base10_parse::<usize>()
            .is_ok_and(|value| !largest.allows(value))
            .then(|| {
                Violation::new(
                    Site::Line(self.path.into(), LineNumber::from(integer.span())),
                    Finding::LargeLiteral {
                        literal: integer.clone(),
                        most: largest,
                    },
                )
            });
        self.budget.violations.extend(finding);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let finding = (self.scope == Scope::OutsideTest && TestCounts::is_assertion(invocation))
            .then(|| {
                Violation::new(
                    Site::Line(self.path.into(), LineNumber::from(invocation.span())),
                    Finding::AssertionOutsideTest,
                )
            });
        self.budget.violations.extend(finding);
        MacroArguments::parse(invocation).visit_with(self);
    }
}
