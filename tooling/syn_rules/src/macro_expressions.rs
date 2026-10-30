use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Expr, Macro, Token};

#[derive(Clone, Debug)]
pub struct MacroExpressions(Punctuated<Expr, Token![,]>);

impl MacroExpressions {
    #[must_use]
    pub fn parsed(invocation: &Macro) -> Option<MacroExpressions> {
        invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ok().map(MacroExpressions)
    }

    pub fn visit_parsed<Visitor: for<'any> Visit<'any>>(invocation: &Macro, visitor: &mut Visitor) {
        let MacroExpressions(arguments) = Self::parsed(invocation).unwrap_or(MacroExpressions(Punctuated::new()));
        arguments.iter().for_each(|argument| visitor.visit_expr(argument));
    }

    pub fn visit_with<'ast, Visitor: Visit<'ast>>(&'ast self, visitor: &mut Visitor) {
        self.0.iter().for_each(|argument| visitor.visit_expr(argument));
    }
}
