use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodName(Ident);

impl MethodName {
    #[must_use]
    pub fn new(ident: &Ident) -> MethodName {
        MethodName(ident.clone())
    }
}
