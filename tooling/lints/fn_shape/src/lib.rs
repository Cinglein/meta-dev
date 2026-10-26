mod finding;

use finding::Finding;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Arm, BinOp, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, FnArg,
    ItemMod, Lit, Local, Pat, Signature, Type, UnOp,
};

use task::LineNumber;
use task::NestingDepth;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;
use task::{Limit, Policy};
use task::{Name, Prefix, Whole};

#[derive(Debug)]
pub struct FnShape;

impl FnShape {
    const BOOLEAN_METHODS: &[&str] = &[
        "contains", "all", "any", "starts_with", "ends_with", "eq", "ne",
    ];
    const BOOLEAN_PREFIXES: [&str; 2] = ["is_", "has_"];

    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "hairy control flow; booleans and tuples become types, nesting becomes methods",
            files
                .iter()
                .flat_map(|file| Self::violations(file, policy))
                .collect(),
        )
    }

    fn violations(file: &SourceFile, policy: &Policy) -> Vec<Violation> {
        let mut shapes = Shapes {
            path: file.path(),
            policy,
            depth: NestingDepth::default(),
            violations: Vec::new(),
        };
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
            Expr::MethodCall(call) => {
                let name = Name::from(&call.method);
                name.matches::<Prefix>(&Self::BOOLEAN_PREFIXES)
                    || name.matches::<Whole>(Self::BOOLEAN_METHODS)
            }
            _ => false,
        }
    }

    fn binds_bool(local: &Local) -> bool {
        matches!(
            &local.pat,
            Pat::Type(typed) if matches!(&*typed.ty, Type::Path(path) if path.path.is_ident("bool"))
        ) || local
            .init
            .as_ref()
            .is_some_and(|init| Self::is_boolean(&init.expr))
    }
}

struct Shapes<'scan> {
    path: &'scan Path,
    policy: &'scan Policy,
    depth: NestingDepth,
    violations: Vec<Violation>,
}

impl Shapes<'_> {
    fn found(path: &Path, span: Span, finding: Finding) -> Violation {
        Violation::new(Site::Line(path.into(), LineNumber::from(span)), finding)
    }

    fn nested(&mut self, span: Span, visit: impl FnOnce(&mut Self)) {
        self.depth += NestingDepth::LEVEL;
        let first_past =
            self.policy.limit(Limit::Nesting).counted::<NestingDepth>() + NestingDepth::LEVEL;
        let deepest = (self.depth == first_past).then_some(span);
        self.violations.extend(deepest.map(|span| {
            Self::found(
                self.path,
                span,
                Finding::DeepNesting {
                    most: self.policy.limit(Limit::Nesting),
                },
            )
        }));
        visit(self);
        self.depth -= NestingDepth::LEVEL;
    }
}

impl<'ast> Visit<'ast> for Shapes<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        let wide = (signature
            .inputs
            .iter()
            .filter(|argument| matches!(argument, FnArg::Typed(_)))
            .count()
            > self.policy.limit(Limit::Parameters).count())
        .then_some(signature.ident.span());
        self.violations.extend(wide.map(|span| {
            Self::found(
                self.path,
                span,
                Finding::WideSignature {
                    most: self.policy.limit(Limit::Parameters),
                },
            )
        }));
        syn::visit::visit_signature(self, signature);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        self.violations.extend(
            FnShape::binds_bool(local)
                .then_some(local.span())
                .map(|span| Self::found(self.path, span, Finding::BooleanLocal)),
        );
        self.violations.extend(
            local
                .init
                .as_ref()
                .filter(|init| matches!(*init.expr, Expr::Tuple(_)))
                .map(|_| Self::found(self.path, local.span(), Finding::TupleLocal)),
        );
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.violations.extend(
            call.args
                .first()
                .filter(|seed| matches!(seed, Expr::Tuple(_)))
                .filter(|_| call.method == "fold" || call.method == "try_fold")
                .map(|_| Self::found(self.path, call.method.span(), Finding::TupleAccumulator)),
        );
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
        arm.guard
            .iter()
            .map(|(_, guard)| guard)
            .for_each(|guard| self.visit_expr(guard));
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
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const SOURCE: &str = "
impl Wide { fn four(a: u8, b: u8, c: u8, d: u8) {}
    fn five(a: u8, b: u8, c: u8, d: u8, e: u8) {}
    fn flags(&self, x: u8) {
        let ready = self.count() > 1; let (lo, hi) = (1, 2);
        let total = self.items.iter().fold((0, 0), |(a, b), x| (a + x, b)); let plain = self.items.iter().fold(0, |a, x| a + x);
        let placed = self.place((1, 2)); let missing = !self.ready(); let inner = (x > 1);
        let empty = self.items.is_empty(); let owns = self.has_key();
        while self.busy() { let idle = false; } loop { let (p, q) = (1, 2); }
        let sizes = self.items.iter().map(|y| { let big = y > 1; big });
        for y in self.items { if y > 1 {
            match y { 2 => self.go(), _ => {} } } }
    }
}
mod inner { fn deep(&self) { let flag = true; } }
#[cfg(test)] mod tests { fn deep(&self) { let flag = true; } }
";
    const REPORTED: [&str; 14] = [
        ":3: more than 4", ":5: boolean local", ":5: tuple literal", ":6: tuple accumulator",
        ":7: boolean local", ":7: boolean local", ":8: boolean local", ":8: boolean local",
        ":9: boolean local", ":9: tuple literal", ":10: boolean local", ":12: control flow nested",
        ":12: control flow nested", ":15: boolean local",
    ];

    #[test]
    fn reports_wide_signatures_boolean_and_tuple_locals_and_deep_nesting_outside_tests() {
        let file = SourceFile::parse("wide.rs", SOURCE.to_owned()).unwrap();
        let report = FnShape::report(core::slice::from_ref(&file), &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
