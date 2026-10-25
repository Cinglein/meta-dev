use std::fmt;

use quote::ToTokens;
use syn::{Field, Ident};

#[derive(Clone, Copy)]
pub struct PublicField<'item> {
    field: &'item Field,
    index: usize,
    owner: &'item Ident,
}

impl<'item> PublicField<'item> {
    pub fn at_index(field: &'item Field, index: usize, owner: &'item Ident) -> PublicField<'item> {
        PublicField { field, index, owner }
    }
}

impl fmt::Display for PublicField<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "field {} of {} is {}; struct fields are private",
            self.field.ident.as_ref().map_or_else(|| self.index.to_string(), ToString::to_string),
            self.owner,
            self.field.vis.to_token_stream().to_string().replace(' ', "")
        )
    }
}
