mod camel_name;

use syn::visit::Visit;
use syn::{Block, ExprLit, ExprPath, Macro};

use super::super::super::TestBudget;
use super::super::super::amount::Amount;
use super::super::super::measurement::Measurement;
use super::macro_arguments::MacroArguments;
use camel_name::CamelName;

#[derive(Default)]
pub(super) struct TestCounts {
    assertions: Amount,
    literals: Amount,
    body_lines: Amount,
}

impl TestCounts {
    pub(super) fn measurements(&self) -> [Measurement; 3] {
        [
            Measurement::new("assertions", self.assertions, TestBudget::MAX_ASSERTIONS_PER_TEST),
            Measurement::new("body lines", self.body_lines, TestBudget::MAX_LINES_PER_TEST),
            Measurement::new("literals", self.literals, TestBudget::MAX_LITERALS_PER_TEST),
        ]
    }

    const ASSERTIONS: [&str; 6] =
        ["assert", "assert_eq", "assert_ne", "prop_assert", "prop_assert_eq", "prop_assert_ne"];

    pub(super) fn tally(block: &Block) -> Self {
        let mut counts = TestCounts { body_lines: Amount::lines_inside(block), ..Self::default() };
        counts.visit_block(block);
        counts
    }

    pub(super) fn is_assertion(invocation: &Macro) -> bool {
        invocation
            .path
            .segments
            .last()
            .is_some_and(|segment| Self::ASSERTIONS.iter().any(|assertion| segment.ident == assertion))
    }

    fn is_variant_path(path: &ExprPath) -> bool {
        path.path.segments.iter().rev().take(2).filter(|segment| CamelName::try_from(&segment.ident).is_ok()).count()
            == 2
    }
}

impl<'ast> Visit<'ast> for TestCounts {
    fn visit_expr_lit(&mut self, literal: &'ast ExprLit) {
        self.literals = self.literals + Amount::ONE;
        syn::visit::visit_expr_lit(self, literal);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.literals = self.literals + Amount::from(Self::is_variant_path(path));
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.assertions = self.assertions + Amount::from(Self::is_assertion(invocation));
        MacroArguments::parse(invocation).visit_with(self);
    }
}
