use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Block, Expr, ExprForLoop, ExprMethodCall, ItemMod, Local, Pat, Stmt};

use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct ManualIteration;

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [&str; 16] = [
        "iter",
        "iter_mut",
        "into_iter",
        "chars",
        "bytes",
        "lines",
        "split",
        "split_whitespace",
        "rev",
        "skip",
        "take",
        "zip",
        "enumerate",
        "map",
        "filter",
        "peekable",
    ];
    const ACCUMULATORS: [&str; 4] = ["push", "extend", "insert", "push_str"];

    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "iterators are driven by combinators, not by hand",
            files.iter().flat_map(Self::violations).collect(),
        )
    }

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut iterations = Iterations { path: file.path(), violations: Vec::new() };
        iterations.visit_file(file.syntax());
        iterations.violations
    }

    fn accumulates(block: &Block) -> bool {
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::If(branch), _)] if branch.else_branch.is_none() => Self::accumulates(&branch.then_branch),
            [Stmt::Expr(Expr::MethodCall(call), _)] => {
                Self::ACCUMULATORS.iter().any(|accumulator| call.method == accumulator)
                    && matches!(*call.receiver, Expr::Path(_) | Expr::Field(_))
            }
            _ => false,
        }
    }
}

struct Iterations<'scan> {
    path: &'scan str,
    violations: Vec<Violation>,
}

impl Iterations<'_> {
    fn report(&mut self, span: Span, message: &str) {
        self.violations.push(Violation::new(Site::spanned(self.path, span), message));
    }
}

impl<'ast> Visit<'ast> for Iterations<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if matches!(&local.pat, Pat::Ident(binding) if binding.mutability.is_some())
            && local.init.as_ref().is_some_and(|init| {
                let mut calls = IteratorCalls::default();
                calls.visit_expr(&init.expr);
                calls.0 > 0
            })
        {
            self.report(
                local.span(),
                "mutable iterator local; drive it with combinators such as format, collect, or fold",
            );
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        if ManualIteration::accumulates(&expr.body) {
            self.report(expr.for_token.span, "loop that only accumulates; extend or collect from an iterator instead");
        }
        syn::visit::visit_expr_for_loop(self, expr);
    }
}

#[derive(Default)]
struct IteratorCalls(usize);

impl<'ast> Visit<'ast> for IteratorCalls {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if ManualIteration::ITERATOR_CONSTRUCTORS.iter().any(|constructor| call.method == constructor) {
            self.0 += 1;
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

#[cfg(test)]
mod tests {
    use super::ManualIteration;
    use crate::source_file::SourceFile;

    const SOURCE: &str = "
impl W {
    fn walk(&self, out: &mut Vec<u8>) {
        let mut items = self.items.iter().rev();
        for item in self.items { out.push(item); }
        for item in self.items { if item > 1 { out.push(item); } }
        for item in self.items { self.visit(item); }
    }
}
";
    const REPORTED: [&str; 3] =
        [":4: mutable iterator local", ":5: loop that only accumulates", ":6: loop that only accumulates"];

    #[test]
    fn flags_mutable_iterator_locals_and_accumulating_loops() {
        let file = SourceFile::parse("walk.rs".to_owned(), SOURCE.to_owned()).expect("valid rust");
        let report = ManualIteration::report(core::slice::from_ref(&file)).to_string();
        assert!(REPORTED.iter().all(|line| report.contains(line)) && !report.contains(":7:"), "{report}");
    }
}
