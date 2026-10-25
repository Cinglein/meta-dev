use std::iter;

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::visit::Visit;
use syn::{Block, Expr, Pat, Token, Type, parenthesized};

pub(in super::super) struct ProptestClosure {
    strategies: Vec<Expr>,
    body: Block,
}

impl ProptestClosure {
    pub(super) fn parse(tokens: TokenStream) -> Option<ProptestClosure> {
        syn::parse2(tokens).ok()
    }

    pub(super) fn visit_with<'ast, V: Visit<'ast>>(&'ast self, visitor: &mut V) {
        self.strategies
            .iter()
            .for_each(|strategy| visitor.visit_expr(strategy));
        visitor.visit_block(&self.body);
    }

    fn strategies(parameters: ParseStream<'_>) -> syn::Result<Vec<Expr>> {
        iter::from_fn(|| (!parameters.is_empty()).then(|| Self::parameter(parameters)))
            .filter_map(Result::transpose)
            .collect()
    }

    fn parameter(parameters: ParseStream<'_>) -> syn::Result<Option<Expr>> {
        Pat::parse_single(parameters)?;
        let strategy = if parameters.peek(Token![in]) {
            parameters.parse::<Token![in]>()?;
            Some(parameters.parse::<Expr>()?)
        } else {
            parameters.parse::<Token![:]>()?;
            parameters.parse::<Type>()?;
            None
        };
        parameters.parse::<Option<Token![,]>>()?;
        Ok(strategy)
    }
}

impl Parse for ProptestClosure {
    fn parse(input: ParseStream<'_>) -> syn::Result<ProptestClosure> {
        input.parse::<Token![|]>()?;
        let parameters;
        parenthesized!(parameters in input);
        let strategies = Self::strategies(&parameters)?;
        input.parse::<Token![|]>()?;
        Ok(ProptestClosure {
            strategies,
            body: input.parse()?,
        })
    }
}
