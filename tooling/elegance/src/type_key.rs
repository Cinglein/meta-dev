use std::fmt;

use quote::ToTokens;
use syn::{Ident, Type};
use task::CrateName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeKey(String);

impl TypeKey {
    #[must_use]
    pub fn local(owner: &CrateName, ident: &Ident) -> TypeKey {
        TypeKey(format!("{owner}::{ident}"))
    }

    #[must_use]
    pub fn foreign(ident: &Ident) -> TypeKey {
        TypeKey(ident.to_string())
    }

    #[must_use]
    pub fn shaped(ty: &Type) -> TypeKey {
        TypeKey(ty.to_token_stream().to_string())
    }
}

impl fmt::Display for TypeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
