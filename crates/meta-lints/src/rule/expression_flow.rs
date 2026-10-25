use std::collections::BTreeMap;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    BinOp, Block, Expr, ExprAssign, ExprBinary, ExprBreak, ExprContinue, ExprForLoop, ExprIf, ExprLoop, ExprReturn,
    ExprWhile, Ident, ImplItemConst, ImplItemFn, ItemConst, ItemFn, ItemMod, Local, Pat, ReturnType, Stmt, TraitItemFn,
};

use crate::region::Region;
use crate::rule::name::Name;
use crate::rule::report::Report;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct ExpressionFlow;

impl ExpressionFlow {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "control flow is an expression; every branch yields the value the function returns",
            files.iter().filter(|file| file.region() == Region::Product).flat_map(Self::violations).collect(),
        )
    }

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut flows = Flows { path: file.path(), yields: Yields::Value, violations: Vec::new() };
        flows.visit_file(file.syntax());
        flows.violations
    }

    fn is_dropped(statement: &Stmt, last: bool) -> bool {
        match statement {
            Stmt::Expr(Expr::If(_) | Expr::Match(_), semicolon) => semicolon.is_some() || !last,
            _ => false,
        }
    }

    fn assigned_local(place: &Expr) -> Option<&Ident> {
        match place {
            Expr::Path(path) => path.path.get_ident().filter(|ident| *ident != "self"),
            Expr::Field(field) => Self::assigned_local(&field.base),
            Expr::Index(index) => Self::assigned_local(&index.expr),
            Expr::Paren(paren) => Self::assigned_local(&paren.expr),
            _ => None,
        }
    }

    fn is_compound(op: BinOp) -> bool {
        matches!(
            op,
            BinOp::AddAssign(_)
                | BinOp::SubAssign(_)
                | BinOp::MulAssign(_)
                | BinOp::DivAssign(_)
                | BinOp::RemAssign(_)
                | BinOp::BitXorAssign(_)
                | BinOp::BitAndAssign(_)
                | BinOp::BitOrAssign(_)
                | BinOp::ShlAssign(_)
                | BinOp::ShrAssign(_)
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Yields {
    Value,
    Nothing,
}

impl Yields {
    fn returned_by(output: &ReturnType) -> Yields {
        match output {
            ReturnType::Default => Yields::Nothing,
            ReturnType::Type(..) => Yields::Value,
        }
    }
}

struct Flows<'scan> {
    path: &'scan str,
    yields: Yields,
    violations: Vec<Violation>,
}

impl Flows<'_> {
    fn report(&mut self, span: Span, message: &str) {
        self.violations.push(Violation::new(Site::spanned(self.path, span), message));
    }

    fn visit_body(&mut self, body: &Block, output: &ReturnType) {
        let outer = self.yields;
        self.yields = Yields::returned_by(output);
        let mut locals = MutableLocals::default();
        locals.visit_block(body);
        locals.only_reassigned().into_iter().for_each(|span| {
            self.report(
                span,
                "mutable local that is only reassigned; a mutable binding is borrowed as &mut or has methods called on it",
            )
        });
        self.visit_block(body);
        self.yields = outer;
    }

    fn report_dropped(&mut self, block: &Block) {
        let last = block.stmts.len().saturating_sub(1);
        let dropped: Vec<Span> = block
            .stmts
            .iter()
            .enumerate()
            .filter(|(index, statement)| ExpressionFlow::is_dropped(statement, *index == last))
            .map(|(_, statement)| statement.span())
            .collect();
        dropped.into_iter().for_each(|span| {
            self.report(span, "if or match whose value is dropped; let it produce the value the function returns")
        });
    }
}

impl<'ast> Visit<'ast> for Flows<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if function.sig.constness.is_none() {
            self.visit_body(&function.block, &function.sig.output);
        }
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        if function.sig.constness.is_none() {
            self.visit_body(&function.block, &function.sig.output);
        }
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        function
            .default
            .as_ref()
            .filter(|_| function.sig.constness.is_none())
            .into_iter()
            .for_each(|body| self.visit_body(body, &function.sig.output));
    }

    fn visit_block(&mut self, block: &'ast Block) {
        if self.yields == Yields::Value {
            self.report_dropped(block);
        }
        syn::visit::visit_block(self, block);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        if self.yields == Yields::Value && expr.else_branch.is_none() {
            self.report(
                expr.if_token.span,
                "if without else yields nothing; use then_some, filter, or an else branch that produces the value",
            );
        }
        syn::visit::visit_expr_if(self, expr);
    }

    fn visit_expr_return(&mut self, expr: &'ast ExprReturn) {
        self.report(
            expr.return_token.span,
            "early return; make the value the tail expression or exit through ? on a Result",
        );
        syn::visit::visit_expr_return(self, expr);
    }

    fn visit_expr_break(&mut self, expr: &'ast ExprBreak) {
        self.report(expr.break_token.span, "break; fold with ControlFlow or find the element with an iterator");
        syn::visit::visit_expr_break(self, expr);
    }

    fn visit_expr_continue(&mut self, expr: &'ast ExprContinue) {
        self.report(expr.continue_token.span, "continue; filter the iterator instead");
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.report(expr.loop_token.span, "loop; drive an iterator with combinators, const fns excepted");
        syn::visit::visit_expr_loop(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.report(expr.for_token.span, "for loop; drive the iterator with combinators, const fns excepted");
        syn::visit::visit_expr_for_loop(self, expr);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.report(expr.while_token.span, "while loop; drive an iterator with combinators, const fns excepted");
        syn::visit::visit_expr_while(self, expr);
    }

    fn visit_expr_assign(&mut self, expr: &'ast ExprAssign) {
        if ExpressionFlow::assigned_local(&expr.left).is_some() {
            self.report(
                expr.eq_token.span,
                "assignment to a local; build the value in one expression or give its type an updater method",
            );
        }
        syn::visit::visit_expr_assign(self, expr);
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        if ExpressionFlow::is_compound(expr.op) && ExpressionFlow::assigned_local(&expr.left).is_some() {
            self.report(
                expr.op.span(),
                "compound assignment to a local; build the value in one expression or give its type an updater method",
            );
        }
        syn::visit::visit_expr_binary(self, expr);
    }
}

#[derive(Default)]
struct MutableLocals {
    declared: BTreeMap<Name, Span>,
    touched: Vec<Name>,
}

impl MutableLocals {
    fn only_reassigned(&self) -> Vec<Span> {
        self.declared.iter().filter(|(name, _)| !self.touched.contains(*name)).map(|(_, span)| *span).collect()
    }
}

impl<'ast> Visit<'ast> for MutableLocals {
    fn visit_local(&mut self, local: &'ast Local) {
        self.declared.extend(match &local.pat {
            Pat::Ident(binding) if binding.mutability.is_some() => {
                Some((Name::from(&binding.ident), local.let_token.span))
            }
            _ => None,
        });
        syn::visit::visit_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        let touched = match expr {
            Expr::MethodCall(call) => ExpressionFlow::assigned_local(&call.receiver),
            Expr::Reference(reference) if reference.mutability.is_some() => {
                ExpressionFlow::assigned_local(&reference.expr)
            }
            _ => None,
        };
        self.touched.extend(touched.map(Name::from));
        syn::visit::visit_expr(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::ExpressionFlow;
    use crate::source_file::SourceFile;

    const SOURCE: &str = "
impl Flow {
    fn guards(&self, items: &[u8]) -> u8 {
        if items.is_empty() { return 0; }
        for item in items { self.note(item); }
        match items.len() { 0 => self.note(0), _ => self.note(1) };
        let mut total = 0;
        total += items.len();
        let mut sorted = items.to_vec();
        sorted.sort();
        if total > 1 { self.note(2) } else { self.note(3) }
    }
    const fn walk(mut rest: &[u8]) -> u8 { while let [head, tail @ ..] = rest { rest = tail; } 0 }
    fn note(&self, item: u8) { if item > 1 { self.log(item); } }
}
";
    const REPORTED: [&str; 6] = [
        ":4: if without else",
        ":4: early return",
        ":5: for loop",
        ":6: if or match whose value is dropped",
        ":7: mutable local that is only reassigned",
        ":8: compound assignment to a local",
    ];
    const CLEAN: [&str; 4] = [":9:", ":11:", ":13:", ":14:"];

    #[test]
    fn reports_returns_loops_dropped_branches_and_reassigned_locals_outside_const_fns() {
        let file = SourceFile::parse("crates/flow.rs".to_owned(), SOURCE.to_owned()).expect("valid rust");
        let report = ExpressionFlow::report(core::slice::from_ref(&file)).to_string();
        assert!(
            REPORTED.iter().all(|line| report.contains(line)) && CLEAN.iter().all(|line| !report.contains(line)),
            "{report}"
        );
    }
}
