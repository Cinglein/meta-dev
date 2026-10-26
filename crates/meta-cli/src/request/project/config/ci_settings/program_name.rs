use std::ffi::OsStr;

use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct ProgramName(String);

impl AsRef<OsStr> for ProgramName {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}
