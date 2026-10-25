use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BooleanMethod;

impl BooleanMethod {
    const PREFIXES: [&str; 2] = ["is_", "has_"];
    const NAMES: [&str; 7] = ["contains", "all", "any", "starts_with", "ends_with", "eq", "ne"];
}

impl TryFrom<&Ident> for BooleanMethod {
    type Error = Name;

    fn try_from(method: &Ident) -> Result<BooleanMethod, Name> {
        let name = method.to_string();
        (Self::PREFIXES.iter().any(|prefix| name.starts_with(prefix)) || Self::NAMES.contains(&name.as_str()))
            .then_some(BooleanMethod)
            .ok_or_else(|| Name::from(method))
    }
}
