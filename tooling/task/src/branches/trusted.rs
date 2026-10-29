use super::BranchName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trusted(Vec<BranchName>);

impl Trusted {
    #[must_use]
    pub fn new(commits: Vec<BranchName>) -> Trusted {
        Trusted(commits)
    }

    #[must_use]
    pub fn commits(&self) -> &[BranchName] {
        &self.0
    }
}
