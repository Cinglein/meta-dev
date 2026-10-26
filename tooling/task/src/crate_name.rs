use std::convert::Infallible;
use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CrateName(String);

impl FromStr for CrateName {
    type Err = Infallible;

    fn from_str(name: &str) -> Result<CrateName, Infallible> {
        Ok(CrateName(name.to_owned()))
    }
}

impl From<&Ident> for CrateName {
    fn from(ident: &Ident) -> CrateName {
        CrateName(ident.to_string())
    }
}
