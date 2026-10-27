use super::sha::Sha;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tip(Option<Sha>);

impl Tip {
    pub fn new(sha: Option<Sha>) -> Self {
        Self(sha)
    }

    pub fn sha(self) -> Option<Sha> {
        self.0
    }
}
