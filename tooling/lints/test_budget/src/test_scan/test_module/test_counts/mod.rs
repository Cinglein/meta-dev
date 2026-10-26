mod assertion_count;
mod literal_count;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Block, ExprLit, ExprPath, Macro};

use super::macro_arguments::MacroArguments;
use crate::line_count::LineCount;
use crate::measurement::Measurement;
use assertion_count::AssertionCount;
use literal_count::LiteralCount;
use task::{Casing, Name, Whole};
use task::{Limit, Policy};

#[derive(Default)]
pub(super) struct TestCounts {
    assertions: AssertionCount,
    literals: LiteralCount,
    body_lines: LineCount,
}

impl TestCounts {
    pub(super) fn overruns(&self, policy: &Policy) -> Vec<String> {
        [
            self.assertions.measured(policy).overrun(),
            Measurement::new(
                "body lines",
                self.body_lines,
                policy.limit(Limit::LinesPerTest).counted(),
            )
            .overrun(),
            self.literals.measured(policy).overrun(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    const ASSERTIONS: &[&str] = &[
        "assert", "assert_eq", "assert_ne", "prop_assert", "prop_assert_eq", "prop_assert_ne",
    ];

    pub(super) fn tally(block: &Block) -> Self {
        let span = block.span();
        let mut counts = TestCounts {
            body_lines: LineCount::from((span.end().line - span.start().line).saturating_sub(1)),
            ..Self::default()
        };
        counts.visit_block(block);
        counts
    }

    pub(super) fn is_assertion(invocation: &Macro) -> bool {
        invocation
            .path
            .segments
            .last()
            .is_some_and(|segment| Name::from(&segment.ident).matches::<Whole>(Self::ASSERTIONS))
    }

    fn is_variant_path(path: &ExprPath) -> bool {
        matches!(
            path.path.segments.iter().rev().collect::<Vec<_>>().as_slice(),
            [variant, parent, ..]
                if Name::from(&variant.ident).casing() == Casing::Camel
                    && Name::from(&parent.ident).casing() == Casing::Camel
        )
    }
}

impl<'ast> Visit<'ast> for TestCounts {
    fn visit_expr_lit(&mut self, literal: &'ast ExprLit) {
        self.literals += LiteralCount::from(1);
        syn::visit::visit_expr_lit(self, literal);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.literals += LiteralCount::from(usize::from(Self::is_variant_path(path)));
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.assertions += AssertionCount::from(usize::from(Self::is_assertion(invocation)));
        MacroArguments::parse(invocation).visit_with(self);
    }
}
