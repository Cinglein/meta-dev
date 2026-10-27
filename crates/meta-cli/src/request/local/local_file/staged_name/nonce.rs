use std::fmt;
use std::hash::{BuildHasher as _, RandomState};

#[derive(Clone, Copy, Debug)]
pub struct Nonce(u64);

impl Nonce {
    pub fn random() -> Self {
        Self(RandomState::new().hash_one(std::time::SystemTime::now()))
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}
