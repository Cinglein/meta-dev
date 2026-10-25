use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CamelName;

impl TryFrom<&Ident> for CamelName {
    type Error = Name;

    fn try_from(ident: &Ident) -> Result<CamelName, Name> {
        let name = Name::from(ident);
        let text = name.as_str();
        if text.starts_with(|letter: char| letter.is_ascii_uppercase())
            && text.chars().any(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit())
        {
            Ok(CamelName)
        } else {
            Err(name)
        }
    }
}
