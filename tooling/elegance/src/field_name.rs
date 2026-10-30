use std::fmt;

use syn::Member;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldName(String);

impl FieldName {
    #[must_use]
    pub fn new(member: &Member) -> FieldName {
        FieldName(match member {
            Member::Named(ident) => ident.to_string(),
            Member::Unnamed(index) => index.index.to_string(),
        })
    }
}

impl fmt::Display for FieldName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
