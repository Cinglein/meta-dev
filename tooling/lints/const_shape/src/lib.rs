use proc_macro2::Span;
use std::path::Path;
use syn::visit::Visit;

use syn::{
    Expr, ExprCast, ExprConst, ExprForLoop, ExprLoop, ExprWhile, ImplItemConst, ImplItemFn,
    ItemConst, ItemFn, ItemMod,
};

use task::LineNumber;
use task::NestingDepth;
use task::Policy;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;

#[derive(Debug)]
pub struct ConstShape;

impl ConstShape {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "casts live in const fns or boundary files, and const loops walk slices",
            files
                .iter()
                .flat_map(|file| Self::violations(file, policy))
                .collect(),
        )
    }

    fn violations(file: &SourceFile, policy: &Policy) -> Vec<Violation> {
        let mut contexts = ConstContexts {
            path: file.path(),
            policy,
            const_depth: NestingDepth::default(),
            violations: Vec::new(),
        };
        contexts.visit_file(file.syntax());
        contexts.violations
    }
}

struct ConstContexts<'scan> {
    path: &'scan Path,
    policy: &'scan Policy,
    const_depth: NestingDepth,
    violations: Vec<Violation>,
}

impl ConstContexts<'_> {
    fn violation(&self, span: Span, message: &str) -> Violation {
        Violation::new(
            Site::Line(self.path.into(), LineNumber::from(span)),
            message,
        )
    }

    fn within_const(&mut self, visit: impl FnOnce(&mut Self)) {
        self.const_depth += NestingDepth::LEVEL;
        visit(self);
        self.const_depth -= NestingDepth::LEVEL;
    }

    fn cast_finding(&self, cast: &ExprCast) -> Option<Violation> {
        (self.const_depth.is_surface() && !self.policy.bit_boundary().covers(self.path)).then(|| {
            self.violation(
                cast.as_token.span,
                "as cast outside a const fn or a bit boundary file; use From, TryFrom, or an EnumMap",
            )
        })
    }

    fn loop_finding(&self, span: Span, message: &str) -> Option<Violation> {
        (!self.const_depth.is_surface()).then(|| self.violation(span, message))
    }
}

impl<'ast> Visit<'ast> for ConstContexts<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if function.sig.constness.is_some() {
            self.within_const(|this| syn::visit::visit_item_fn(this, function));
        } else {
            syn::visit::visit_item_fn(self, function);
        }
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        if function.sig.constness.is_some() {
            self.within_const(|this| syn::visit::visit_impl_item_fn(this, function));
        } else {
            syn::visit::visit_impl_item_fn(self, function);
        }
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.within_const(|this| syn::visit::visit_item_const(this, item));
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.within_const(|this| syn::visit::visit_impl_item_const(this, item));
    }

    fn visit_expr_const(&mut self, block: &'ast ExprConst) {
        self.within_const(|this| syn::visit::visit_expr_const(this, block));
    }

    fn visit_expr_cast(&mut self, cast: &'ast ExprCast) {
        let finding = self.cast_finding(cast);
        self.violations.extend(finding);
        syn::visit::visit_expr_cast(self, cast);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        let finding = (!matches!(*expr.cond, Expr::Let(_))).then(|| {
            self.loop_finding(
                expr.while_token.span,
                "counter loop in const context; walk a slice with while let [head, rest @ ..]",
            )
        });
        self.violations.extend(finding.flatten());
        syn::visit::visit_expr_while(self, expr);
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        let finding = self.loop_finding(
            expr.loop_token.span,
            "open loop in const context; walk a slice with while let or recurse",
        );
        self.violations.extend(finding);
        syn::visit::visit_expr_loop(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        let finding = self.loop_finding(expr.for_token.span, "for loop in const context");
        self.violations.extend(finding);
        syn::visit::visit_expr_for_loop(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::ConstShape;
    use settings::EmbeddedDefaults;
    use task::Policy;
    use task::SourceFile;
    const BIT_BOUNDARY: &str = "bit_boundary";
    const SOURCE: &str = "
const fn free(mut rest: &[u8]) -> u8 { let mut i = 0; while i < 2 { i += 1; } while let [_, tail @ ..] = rest { rest = tail; } loop { break i } }
const CELLS: u8 = { let mut i = 0; while i < 2 { i += 1; } i };
impl Table { const SIZE: u8 = { for n in 0..2 {} 0 }; const fn build(key: u8) -> u8 { key as u8 } }
impl Table { fn lookup(&self, key: Kind) -> u8 { let z = const { loop {} }; let mut i = 0; while i < 2 { i += 1; } for n in 0..2 {} loop { break self.0[key as usize] } } }
mod inner { fn widen(x: u8) -> u16 { x as u16 } }
#[cfg(test)] mod tests { fn widen(x: u8) -> u16 { x as u16 } }
";
    const BOUNDARY: &str = "crates/board/src/bitboard/mod.rs";
    const BITS: &str = "fn widen(x: u8) -> u16 { x as u16 }";
    const REPORTED: [&str; 7] = [
        ":2: counter loop", ":2: open loop", ":3: counter loop", ":4: for loop", ":5: open loop",
        ":5: as cast", ":6: as cast",
    ];

    #[test]
    fn flags_counter_and_open_loops_in_const_context_and_casts_outside_const_or_boundary_files() {
        let mut table: toml::Table = toml::from_str(Policy::DEFAULTS).unwrap();
        table.insert(BIT_BOUNDARY.to_owned(), toml::Value::from(vec![BOUNDARY]));
        let policy: Policy = table.try_into().unwrap();
        let file = SourceFile::parse("table.rs", SOURCE.to_owned()).unwrap();
        let bits = SourceFile::parse(BOUNDARY, BITS.to_owned()).unwrap();
        let report = ConstShape::report(&[bits, file], &policy);
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
