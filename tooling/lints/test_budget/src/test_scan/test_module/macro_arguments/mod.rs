mod proptest_closure;

use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Expr, Macro, Token};

use proptest_closure::ProptestClosure;

pub(super) enum MacroArguments {
    Expressions(Punctuated<Expr, Token![,]>),
    Closure(ProptestClosure),
    Opaque,
}

impl MacroArguments {
    pub(super) fn parse(invocation: &Macro) -> MacroArguments {
        invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map(MacroArguments::Expressions)
            .ok()
            .or_else(|| {
                ProptestClosure::parse(invocation.tokens.clone()).map(MacroArguments::Closure)
            })
            .unwrap_or(MacroArguments::Opaque)
    }

    pub(super) fn visit_with<'ast, V: Visit<'ast>>(&'ast self, visitor: &mut V) {
        match self {
            MacroArguments::Expressions(arguments) => arguments
                .iter()
                .for_each(|argument| visitor.visit_expr(argument)),
            MacroArguments::Closure(closure) => closure.visit_with(visitor),
            MacroArguments::Opaque => {}
        }
    }
}
