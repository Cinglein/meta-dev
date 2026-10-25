use std::fmt;

use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortLifetime(Ident);

impl ShortLifetime {
    const MIN_NAME_LENGTH: usize = 2;

    pub fn name(&self) -> &Ident {
        &self.0
    }
}

impl TryFrom<&Ident> for ShortLifetime {
    type Error = Name;

    fn try_from(name: &Ident) -> Result<ShortLifetime, Name> {
        (name.to_string().len() < Self::MIN_NAME_LENGTH)
            .then(|| ShortLifetime(name.clone()))
            .ok_or_else(|| Name::from(name))
    }
}

impl fmt::Display for ShortLifetime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "'{}", self.0)
    }
}
