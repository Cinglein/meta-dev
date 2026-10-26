mod raised;

use super::Flags;
use crate::flag::Flag;
use crate::github_failure::GithubFailure;
use raised::Raised;

const OFFLINE: &str = "offline";

#[test]
fn since_keeps_only_the_flags_absent_from_the_previous_set_in_order() {
    let now = Flags::new(vec![Raised::Orphan.flag(), Raised::Red.flag(), Raised::Stalled.flag()]);
    let before = Flags::new(vec![Raised::Red.flag()]);
    assert_eq!(now.since(&before), Flags::new(vec![Raised::Orphan.flag(), Raised::Stalled.flag()]));
    assert_eq!(now.since(&now), Flags::default());
    assert_eq!(now.since(&Flags::default()), now);
}

#[test]
fn only_orphans_never_await_judgement() {
    let unreached = Flag::Unreached(GithubFailure::new(OFFLINE));
    let now = Flags::new(vec![Raised::Orphan.flag(), Raised::Red.flag(), Flag::TruncatedOpen, unreached]);
    assert_eq!(now.awaiting_judgement().to_string().lines().count(), now.to_string().lines().count() - 1);
    assert_eq!(Flags::new(vec![Raised::Orphan.flag()]).awaiting_judgement().raised(), None);
}

#[test]
fn flags_survive_a_round_trip_through_json() {
    let now = Flags::new(vec![Raised::Orphan.flag(), Raised::Red.flag(), Raised::Stalled.flag(), Flag::TruncatedOpen]);
    let text = serde_json::to_string(&now).unwrap();
    assert_eq!(serde_json::from_str::<Flags>(&text).unwrap(), now);
}
