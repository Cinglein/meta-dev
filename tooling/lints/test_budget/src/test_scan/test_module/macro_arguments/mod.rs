mod proptest_closure;

use syn::Macro;
use syn::visit::Visit;
use task::MacroExpressions;

use proptest_closure::ProptestClosure;

pub(super) enum MacroArguments {
    Expressions(MacroExpressions),
    Closure(ProptestClosure),
    Opaque,
}

impl MacroArguments {
    pub(super) fn parse(invocation: &Macro) -> MacroArguments {
        MacroExpressions::parsed(invocation)
            .map(MacroArguments::Expressions)
            .or_else(|| {
                ProptestClosure::parse(invocation.tokens.clone()).map(MacroArguments::Closure)
            })
            .unwrap_or(MacroArguments::Opaque)
    }

    pub(super) fn visit_with<'ast, V: Visit<'ast>>(&'ast self, visitor: &mut V) {
        match self {
            MacroArguments::Expressions(arguments) => arguments.visit_with(visitor),
            MacroArguments::Closure(closure) => closure.visit_with(visitor),
            MacroArguments::Opaque => {}
        }
    }
}
