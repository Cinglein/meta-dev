use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{Block, Expr, ExprForLoop, ExprMethodCall, ItemMod, Local, Pat, Stmt};

use task::LineNumber;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;
use task::{Name, Whole};

#[derive(Debug)]
pub struct ManualIteration;

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [&str; 16] = [
        "iter", "iter_mut", "into_iter", "chars", "bytes", "lines", "split", "split_whitespace",
        "rev", "skip", "take", "zip", "enumerate", "map", "filter", "peekable",
    ];
    const ACCUMULATORS: [&str; 4] = ["push", "extend", "insert", "push_str"];

    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "iterators are driven by combinators, not by hand",
            files.iter().flat_map(Self::violations).collect(),
        )
    }

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut iterations = Iterations {
            path: file.path(),
            violations: Vec::new(),
        };
        iterations.visit_file(file.syntax());
        iterations.violations
    }

    fn accumulates(block: &Block) -> bool {
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::If(branch), _)] if branch.else_branch.is_none() => {
                Self::accumulates(&branch.then_branch)
            }
            [Stmt::Expr(Expr::MethodCall(call), _)] => {
                Name::from(&call.method).matches::<Whole>(&Self::ACCUMULATORS)
                    && matches!(*call.receiver, Expr::Path(_) | Expr::Field(_))
            }
            _ => false,
        }
    }
}

struct Iterations<'scan> {
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl Iterations<'_> {
    fn violation(&self, span: Span, message: &str) -> Violation {
        Violation::new(
            Site::Line(self.path.into(), LineNumber::from(span)),
            message,
        )
    }
}

impl<'ast> Visit<'ast> for Iterations<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let finding = (matches!(&local.pat, Pat::Ident(binding) if binding.mutability.is_some())
            && local.init.as_ref().is_some_and(|init| {
                let mut calls = IteratorCalls::default();
                calls.visit_expr(&init.expr);
                calls.0 > 0
            }))
        .then(|| {
            self.violation(
                local.span(),
                "mutable iterator local; drive it with combinators such as format, collect, or fold",
            )
        });
        self.violations.extend(finding);
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        let finding = ManualIteration::accumulates(&expr.body).then(|| {
            self.violation(
                expr.for_token.span,
                "loop that only accumulates; extend or collect from an iterator instead",
            )
        });
        self.violations.extend(finding);
        syn::visit::visit_expr_for_loop(self, expr);
    }
}

#[derive(Default)]
struct IteratorCalls(usize);

impl<'ast> Visit<'ast> for IteratorCalls {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.0 += usize::from(
            Name::from(&call.method).matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS),
        );
        syn::visit::visit_expr_method_call(self, call);
    }
}
