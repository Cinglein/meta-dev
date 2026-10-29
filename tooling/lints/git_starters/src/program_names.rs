use proc_macro2::{Literal, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{BinOp, ExprBinary, LitStr, Macro};

use task::LineNumber;

#[derive(Debug, Default)]
pub(crate) struct ProgramNames(Vec<LineNumber>);

impl ProgramNames {
    const PROGRAM: &str = concat!("gi", "t");

    pub(crate) fn lines(self) -> Vec<LineNumber> {
        self.0
    }

    fn tokens_naming(&mut self, tokens: TokenStream) {
        tokens.into_iter().for_each(|token| match token {
            TokenTree::Group(group) => self.tokens_naming(group.stream()),
            TokenTree::Literal(literal) => self.literal_naming(literal),
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        });
    }

    fn literal_naming(&mut self, literal: Literal) {
        let text = syn::parse2::<LitStr>(TokenStream::from(TokenTree::Literal(literal)));
        text.iter().for_each(|text| self.visit_lit_str(text));
    }
}

impl<'ast> Visit<'ast> for ProgramNames {
    fn visit_expr_binary(&mut self, binary: &'ast ExprBinary) {
        match binary.op {
            BinOp::Eq(_) | BinOp::Ne(_) => {}
            _ => visit::visit_expr_binary(self, binary),
        }
    }

    fn visit_lit_str(&mut self, literal: &'ast LitStr) {
        self.0
            .extend((literal.value() == Self::PROGRAM).then(|| LineNumber::from(literal.span())));
    }

    fn visit_macro(&mut self, called: &'ast Macro) {
        self.tokens_naming(called.tokens.clone());
    }
}
