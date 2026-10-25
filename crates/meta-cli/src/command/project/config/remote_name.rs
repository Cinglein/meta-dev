use std::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct RemoteName(String);

impl fmt::Display for RemoteName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
