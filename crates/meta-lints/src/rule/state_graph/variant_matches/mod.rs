mod foreign_match;
mod variant_paths;

use foreign_match::ForeignMatch;
use variant_paths::VariantPaths;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprLet, ExprMatch, Local, Pat};

use super::declarations::Declarations;
use crate::rule::name::Name;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct VariantMatches<'scan> {
    declarations: &'scan Declarations,
    path: &'scan str,
    violations: Vec<Violation>,
}

impl<'scan> VariantMatches<'scan> {
    pub fn violations(file: &'scan SourceFile, declarations: &'scan Declarations) -> Vec<Violation> {
        let mut matches = VariantMatches { declarations, path: file.path(), violations: Vec::new() };
        matches.visit_file(file.syntax());
        matches.violations
    }

    fn foreign_variant(&self, pat: &Pat) -> Option<Name> {
        VariantPaths::in_pat(pat)
            .into_iter()
            .find(|enum_name| self.declarations.is_foreign_sum_type(enum_name, self.path))
    }

    fn foreign_match(&self, pat: &Pat) -> Option<Violation> {
        self.foreign_variant(pat)
            .map(|enum_name| Violation::new(Site::spanned(self.path, pat.span()), ForeignMatch::new(&enum_name)))
    }

    fn is_table(expr: &Expr) -> bool {
        match expr {
            Expr::Lit(_) | Expr::Path(_) => true,
            Expr::Unary(unary) => Self::is_table(&unary.expr),
            Expr::Paren(paren) => Self::is_table(&paren.expr),
            Expr::Tuple(tuple) => tuple.elems.iter().all(Self::is_table),
            Expr::Array(array) => array.elems.iter().all(Self::is_table),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for VariantMatches<'_> {
    fn visit_expr_match(&mut self, expr: &'ast ExprMatch) {
        let foreign: Vec<Violation> = expr
            .arms
            .iter()
            .filter(|arm| !Self::is_table(&arm.body))
            .filter_map(|arm| self.foreign_match(&arm.pat))
            .collect();
        self.violations.extend(foreign);
        syn::visit::visit_expr_match(self, expr);
    }

    fn visit_expr_let(&mut self, expr: &'ast ExprLet) {
        let foreign = self.foreign_match(&expr.pat);
        self.violations.extend(foreign);
        syn::visit::visit_expr_let(self, expr);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let foreign =
            local.init.as_ref().filter(|init| init.diverge.is_some()).and_then(|_| self.foreign_match(&local.pat));
        self.violations.extend(foreign);
        syn::visit::visit_local(self, local);
    }
}
