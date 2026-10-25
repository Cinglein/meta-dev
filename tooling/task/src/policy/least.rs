use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Least(usize);

impl Least {
    pub const fn allows(self, length: usize) -> bool {
        length >= self.0
    }
}
