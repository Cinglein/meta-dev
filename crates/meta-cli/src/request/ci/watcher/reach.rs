use crate::request::ci::Commit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    Remote,
    Tracking,
}

impl Reach {
    pub fn asks_remote(self, created: &[Commit]) -> Option<()> {
        created.first().filter(|_| self == Self::Remote).map(drop)
    }
}
