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

#[cfg(test)]
mod tests {
    use super::{BranchName, Trusted};
    const COMMIT: &str = "abc123";

    #[test]
    fn a_trusted_list_hands_back_the_commits_it_was_given() {
        let commit: BranchName = COMMIT.parse().unwrap();
        assert_eq!(Trusted::new(vec![commit.clone()]).commits(), [commit]);
    }
}
