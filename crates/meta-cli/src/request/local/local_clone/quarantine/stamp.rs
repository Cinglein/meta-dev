use std::ffi::OsStr;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp(String);

impl Stamp {
    pub fn current() -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_nanos()).unwrap_or_default();
        Self(format!("{}x{nanos}", std::process::id()))
    }
}

impl FromStr for Stamp {
    type Err = std::convert::Infallible;

    fn from_str(stamp: &str) -> Result<Self, Self::Err> {
        Ok(Self(stamp.to_owned()))
    }
}

impl AsRef<OsStr> for Stamp {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}
