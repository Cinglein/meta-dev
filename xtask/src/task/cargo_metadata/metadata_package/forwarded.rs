use std::convert::Infallible;
use std::str::FromStr;

use serde::Deserialize;
use task::CrateName;

#[derive(Debug, Deserialize)]
#[serde(from = "String")]
pub struct Forwarded {
    dependency: Option<CrateName>,
    feature: String,
}

impl Forwarded {
    const FORWARD: char = '/';
    const WEAK: char = '?';

    pub fn local(&self) -> Option<String> {
        self.dependency.is_none().then(|| self.feature.clone())
    }

    pub fn enabling(&self, alias: &CrateName) -> Option<String> {
        (self.dependency.as_ref() == Some(alias)).then(|| self.feature.clone())
    }
}

impl FromStr for Forwarded {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Forwarded, Infallible> {
        let forwarded = value.split_once(Self::FORWARD).map(|(dependency, feature)| Forwarded {
            dependency: dependency.trim_end_matches(Self::WEAK).parse().ok(),
            feature: feature.to_owned(),
        });
        Ok(forwarded.unwrap_or_else(|| Forwarded { dependency: None, feature: value.to_owned() }))
    }
}

impl From<String> for Forwarded {
    fn from(value: String) -> Forwarded {
        let Ok(forwarded) = value.parse();
        forwarded
    }
}
