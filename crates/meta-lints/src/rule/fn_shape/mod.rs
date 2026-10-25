mod arity;
mod boolean_method;
mod depth;

use arity::Arity;
use boolean_method::BooleanMethod;
use depth::Depth;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Arm, BinOp, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, ItemMod, Lit, Local, Pat,
    Signature, Type, UnOp,
};

use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct FnShape;

impl FnShape {
    const MAX_ARITY: Arity = Arity::new(4);
    const MAX_NESTING: Depth = Depth::new(2);

    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "hairy control flow; booleans and tuples become types, nesting becomes methods",
            files.iter().flat_map(Self::violations).collect(),
        )
    }

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut shapes = Shapes { path: file.path(), depth: Depth::default(), violations: Vec::new() };
        shapes.visit_file(file.syntax());
        shapes.violations
    }

    fn is_boolean(expr: &Expr) -> bool {
        match expr {
            Expr::Binary(binary) => matches!(
                binary.op,
                BinOp::Eq(_)
                    | BinOp::Ne(_)
                    | BinOp::Lt(_)
                    | BinOp::Le(_)
                    | BinOp::Gt(_)
                    | BinOp::Ge(_)
                    | BinOp::And(_)
                    | BinOp::Or(_)
            ),
            Expr::Unary(unary) => matches!(unary.op, UnOp::Not(_)),
            Expr::Lit(literal) => matches!(literal.lit, Lit::Bool(_)),
            Expr::Paren(paren) => Self::is_boolean(&paren.expr),
            Expr::MethodCall(call) => BooleanMethod::try_from(&call.method).is_ok(),
            _ => false,
        }
    }

    fn binds_bool(local: &Local) -> bool {
        matches!(
            &local.pat,
            Pat::Type(typed) if matches!(&*typed.ty, Type::Path(path) if path.path.is_ident("bool"))
        ) || local.init.as_ref().is_some_and(|init| Self::is_boolean(&init.expr))
    }
}

struct Shapes<'scan> {
    path: &'scan str,
    depth: Depth,
    violations: Vec<Violation>,
}

impl Shapes<'_> {
    fn report(&mut self, span: Span, message: &str) {
        self.violations.push(Violation::new(Site::spanned(self.path, span), message));
    }

    fn nested(&mut self, span: Span, visit: impl FnOnce(&mut Self)) {
        let outer = self.depth;
        self.depth = outer.deeper();
        if self.depth == FnShape::MAX_NESTING.deeper() {
            self.report(span, "control flow nested deeper than two levels; lift the inner part into a method");
        }
        visit(self);
        self.depth = outer;
    }
}

impl<'ast> Visit<'ast> for Shapes<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        if Arity::counted_in(signature) > FnShape::MAX_ARITY {
            self.report(
                signature.ident.span(),
                "more than four parameters; group the ones that travel together into a struct",
            );
        }
        syn::visit::visit_signature(self, signature);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if FnShape::binds_bool(local) {
            self.report(local.span(), "boolean local; inline a one-use condition or make a mode a type");
        }
        if local.init.as_ref().is_some_and(|init| matches!(*init.expr, Expr::Tuple(_))) {
            self.report(local.span(), "tuple literal bound to a local; name it as a struct");
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if (call.method == "fold" || call.method == "try_fold")
            && call.args.first().is_some_and(|seed| matches!(seed, Expr::Tuple(_)))
        {
            self.report(call.method.span(), "tuple accumulator in a fold; give the accumulator a type");
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.visit_expr(&expr.cond);
        self.nested(expr.if_token.span, |this| {
            this.visit_block(&expr.then_branch);
        });
        match expr.else_branch.as_ref().map(|(_, branch)| &**branch) {
            None => {}
            Some(Expr::If(chained)) => self.visit_expr_if(chained),
            Some(other) => self.nested(other.span(), |this| this.visit_expr(other)),
        }
    }

    fn visit_arm(&mut self, arm: &'ast Arm) {
        self.visit_pat(&arm.pat);
        arm.guard.iter().for_each(|(_, guard)| self.visit_expr(guard));
        self.nested(arm.body.span(), |this| this.visit_expr(&arm.body));
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.visit_expr(&expr.expr);
        self.nested(expr.for_token.span, |this| this.visit_block(&expr.body));
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.visit_expr(&expr.cond);
        self.nested(expr.while_token.span, |this| this.visit_block(&expr.body));
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.nested(expr.loop_token.span, |this| this.visit_block(&expr.body));
    }

    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        self.nested(closure.or1_token.span, |this| {
            this.visit_expr(&closure.body);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::FnShape;
    use crate::source_file::SourceFile;

    const SOURCE: &str = "
impl Wide {
    fn five(a: u8, b: u8, c: u8, d: u8, e: u8) {}
    fn flags(&self) {
        let ready = self.count() > 1;
        let (lo, hi) = (1, 2);
        let total = self.items.iter().fold((0, 0), |(a, b), x| (a + x, b));
        for x in self.items { if x > 1 { match x { 2 => self.go(), _ => {} } } }
    }
}
";
    const REPORTED: [&str; 5] = [
        ":3: more than four parameters",
        ":5: boolean local",
        ":6: tuple literal bound",
        ":7: tuple accumulator",
        ":8: control flow nested deeper",
    ];

    #[test]
    fn reports_wide_signatures_boolean_and_tuple_locals_and_deep_nesting() {
        let file = SourceFile::parse("wide.rs".to_owned(), SOURCE.to_owned()).expect("valid rust");
        let report = FnShape::report(core::slice::from_ref(&file)).to_string();
        assert!(REPORTED.iter().all(|line| report.contains(line)), "{report}");
    }
}
