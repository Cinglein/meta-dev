use std::fmt;

use quote::ToTokens;
use syn::{GenericArgument, Ident, PathArguments, PathSegment, Type};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeName(String);

impl TypeName {
    const WRAPPERS: [&str; 2] = ["Option", "Result"];

    pub fn from_type(ty: &Type) -> TypeName {
        TypeName(Self::unwrapped(ty).to_token_stream().to_string().replace(' ', ""))
    }

    pub fn named(ident: &Ident) -> TypeName {
        TypeName(ident.to_string())
    }

    pub fn or_self(self, self_type: &TypeName) -> TypeName {
        if self.0 == "Self" { self_type.clone() } else { self }
    }

    fn unwrapped(ty: &Type) -> &Type {
        match ty {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .filter(|segment| Self::is_wrapper(segment))
                .and_then(Self::first_type_argument)
                .unwrap_or(ty),
            _ => ty,
        }
    }

    fn is_wrapper(segment: &PathSegment) -> bool {
        Self::WRAPPERS.iter().any(|wrapper| segment.ident == wrapper)
    }

    fn first_type_argument(segment: &PathSegment) -> Option<&Type> {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => match arguments.args.first() {
                Some(GenericArgument::Type(inner)) => Some(inner),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
