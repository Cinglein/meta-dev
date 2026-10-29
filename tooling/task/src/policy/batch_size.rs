use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BatchSize(NonZeroUsize);

impl BatchSize {
    #[must_use]
    pub fn admits<T>(self, members: &[T]) -> bool {
        members.len() < self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::BatchSize;

    #[test]
    fn a_batch_admits_members_until_it_holds_its_size_and_zero_is_refused() {
        let size: BatchSize = toml::from_str::<toml::Table>("size = 2")
            .unwrap()
            .remove("size")
            .unwrap()
            .try_into()
            .unwrap();
        assert!(size.admits(&[()]));
        assert!(!size.admits(&[(), ()]));
        assert!(toml::Value::Integer(0).try_into::<BatchSize>().is_err());
    }
}
