use std::convert::Infallible;
use std::str::FromStr;

use serde::Deserialize;

use crate::invocation::Invocation;

#[derive(Debug, Deserialize)]
#[serde(from = "String")]
pub(crate) struct Sid(Option<Invocation>);

impl Sid {
    const PARENT: char = '/';

    pub(crate) fn invocation(self) -> Option<Invocation> {
        self.0
    }
}

impl FromStr for Sid {
    type Err = Infallible;

    fn from_str(sid: &str) -> Result<Sid, Infallible> {
        Ok(Sid(sid.split_once(Self::PARENT).and_then(|(parent, _)| Invocation::tagged(parent))))
    }
}

impl From<String> for Sid {
    fn from(sid: String) -> Sid {
        let Ok(parsed) = sid.parse();
        parsed
    }
}
