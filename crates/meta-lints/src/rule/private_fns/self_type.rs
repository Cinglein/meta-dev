use std::fmt;

use quote::ToTokens;
use syn::Type;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelfType(String);

impl From<&Type> for SelfType {
    fn from(self_type: &Type) -> SelfType {
        SelfType(self_type.to_token_stream().to_string().replace(' ', ""))
    }
}

impl fmt::Display for SelfType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
