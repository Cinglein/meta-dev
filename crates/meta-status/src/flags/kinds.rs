use super::Flags;
use crate::flag::Flag;
use crate::github_failure::GithubFailure;

const TIMED_OUT: &str = "timed out";
const REFUSED: &str = "connection refused";

#[test]
fn an_unreachable_github_is_the_same_flag_whatever_the_reason() {
    let before = Flags::new(vec![Flag::Unreached(GithubFailure::new(TIMED_OUT))]);
    let now = Flags::new(vec![Flag::Unreached(GithubFailure::new(REFUSED))]);
    assert_eq!(now.since(&before), Flags::default());
    assert_eq!(now.since(&Flags::default()), now);
}
