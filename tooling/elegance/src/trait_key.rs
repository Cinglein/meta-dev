use std::fmt;

use syn::Ident;
use task::CrateName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraitKey(String);

impl TraitKey {
    #[must_use]
    pub fn local(owner: &CrateName, ident: &Ident) -> TraitKey {
        TraitKey(format!("{owner}::{ident}"))
    }

    #[must_use]
    pub fn foreign(ident: &Ident) -> TraitKey {
        TraitKey(ident.to_string())
    }
}

impl fmt::Display for TraitKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
