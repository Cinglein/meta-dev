use std::convert::Infallible;
use std::ffi::OsStr;
use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct RemoteName(String);

impl RemoteName {
    pub(crate) fn tracking_glob(&self) -> String {
        format!("refs/remotes/{}/*", self.0)
    }
}

impl FromStr for RemoteName {
    type Err = Infallible;

    fn from_str(name: &str) -> Result<RemoteName, Infallible> {
        Ok(RemoteName(name.to_owned()))
    }
}

impl AsRef<OsStr> for RemoteName {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl fmt::Display for RemoteName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
