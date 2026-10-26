use derive_more::{Add, Display, From, Into};

use crate::test_count::TestCount;
use task::{Policy, Share};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Add, Display, From, Into)]
pub struct FileCount(usize);

impl FileCount {
    pub fn tests_allowed(self, policy: &Policy) -> TestCount {
        policy.share(Share::TestsPerHundredFiles).applied_to(self)
    }
}
