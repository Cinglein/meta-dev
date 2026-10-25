use std::fmt;

use quote::ToTokens;
use syn::{GenericArgument, Ident, PathArguments, Type};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeName(String);

impl TypeName {
    const WRAPPERS: [&str; 2] = ["Option", "Result"];

    #[must_use]
    pub fn from_type(ty: &Type) -> TypeName {
        TypeName(
            Self::unwrapped(ty)
                .to_token_stream()
                .to_string()
                .replace(' ', ""),
        )
    }

    #[must_use]
    pub fn named(ident: &Ident) -> TypeName {
        TypeName(ident.to_string())
    }

    #[must_use]
    pub fn or_self(self, self_type: &TypeName) -> TypeName {
        if self.0 == "Self" {
            self_type.clone()
        } else {
            self
        }
    }

    fn unwrapped(ty: &Type) -> &Type {
        let last = match ty {
            Type::Path(path) => path.path.segments.last(),
            _ => None,
        };
        last.filter(|segment| Self::WRAPPERS.contains(&segment.ident.to_string().as_str()))
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.first(),
                _ => None,
            })
            .and_then(|argument| match argument {
                GenericArgument::Type(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or(ty)
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
