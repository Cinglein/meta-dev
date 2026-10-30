use std::convert::Infallible;
use std::path::Path;
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

impl From<&Path> for CrateName {
    fn from(binary: &Path) -> CrateName {
        let stem = binary
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        CrateName(
            stem.rsplit_once('-')
                .map_or(stem.as_str(), |(name, _)| name)
                .to_owned(),
        )
    }
}

impl From<&Ident> for CrateName {
    fn from(ident: &Ident) -> CrateName {
        CrateName(ident.to_string())
    }
}
