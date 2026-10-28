use std::collections::BTreeMap;
use std::path::PathBuf;

use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{
    Expr, ExprLit, ExprMethodCall, Ident, ImplItemConst, ItemConst, Lit, LitStr, Macro, Token,
};

use super::finding::Finding;
use task::LineNumber;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::{Name, Whole};

pub struct RefSites {
    path: PathBuf,
    constants: BTreeMap<Name, LitStr>,
    joined: Vec<Ident>,
    found: Vec<Violation>,
}

impl RefSites {
    const GIT_DIRECTORY: &str = ".git";
    const REF_STORES: [&str; 3] = ["HEAD", "refs", "packed-refs"];
    const PSEUDO_REF: &str = "_HEAD";
    const JOINING: &str = "join";

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut sites = RefSites {
            path: file.path().to_owned(),
            constants: BTreeMap::new(),
            joined: Vec::new(),
            found: Vec::new(),
        };
        sites.visit_file(file.syntax());
        let resolved: Vec<Violation> = sites
            .joined
            .iter()
            .filter_map(|ident| {
                sites
                    .constants
                    .get(&Name::from(ident))
                    .filter(|text| Self::names_a_ref_store(text))
                    .map(|text| sites.violation(LineNumber::from(ident.span()), text))
            })
            .collect();
        sites.found.into_iter().chain(resolved).collect()
    }

    fn violation(&self, line: LineNumber, text: &LitStr) -> Violation {
        Violation::new(
            Site::Line(self.path.clone(), line),
            Finding::RefFile {
                path: Name::from(&*text.value()),
            },
        )
    }

    fn names_a_ref_store(text: &LitStr) -> bool {
        text.value().split('/').any(|segment| {
            Name::from(segment).matches::<Whole>(&Self::REF_STORES)
                || segment.ends_with(Self::PSEUDO_REF)
        })
    }

    fn constant(expr: &Expr) -> Option<LitStr> {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Str(text),
                ..
            }) => Some(text.clone()),
            _ => None,
        }
    }

    fn joined_argument(&mut self, argument: &Expr) {
        match (argument, Self::constant(argument)) {
            (Expr::Path(path), _) => self.joined.extend(
                path.path
                    .segments
                    .last()
                    .map(|segment| segment.ident.clone()),
            ),
            (_, Some(text)) => {
                let found = Self::names_a_ref_store(&text)
                    .then(|| self.violation(LineNumber::from(text.span()), &text));
                self.found.extend(found);
            }
            (_, None) => self.visit_expr(argument),
        }
    }
}

impl<'ast> Visit<'ast> for RefSites {
    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.constants
            .extend(Self::constant(&item.expr).map(|text| (Name::from(&item.ident), text)));
        syn::visit::visit_item_const(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.constants
            .extend(Self::constant(&item.expr).map(|text| (Name::from(&item.ident), text)));
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if call.method == Self::JOINING {
            self.visit_expr(&call.receiver);
            call.args
                .iter()
                .for_each(|argument| self.joined_argument(argument));
        } else {
            syn::visit::visit_expr_method_call(self, call);
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .into_iter()
            .flatten()
            .for_each(|argument| self.visit_expr(&argument));
    }

    fn visit_lit_str(&mut self, text: &'ast LitStr) {
        let found = (text
            .value()
            .split('/')
            .any(|segment| segment == Self::GIT_DIRECTORY)
            && Self::names_a_ref_store(text))
        .then(|| self.violation(LineNumber::from(text.span()), text));
        self.found.extend(found);
    }
}
