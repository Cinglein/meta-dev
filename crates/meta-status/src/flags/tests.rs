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
fn only_orphans_never_await_judgement_and_flags_survive_a_round_trip_through_json() {
    let unreached = Flag::Unreached(GithubFailure::new(OFFLINE));
    let stalled = Raised::Stalled.flag();
    let judged = [Raised::Red.flag(), Raised::HeldDone.flag(), Raised::TrunkRed.flag(), Flag::TruncatedOpen, unreached];
    let judged = judged.into_iter().chain([stalled]);
    let now = Flags::new([Raised::Orphan.flag()].into_iter().chain(judged).collect());
    assert_eq!(now.awaiting_judgement().to_string().lines().count(), now.to_string().lines().count() - 1);
    assert_eq!(Flags::new(vec![Raised::Orphan.flag()]).awaiting_judgement().raised(), None);
    let text = serde_json::to_string(&now).unwrap();
    assert_eq!(serde_json::from_str::<Flags>(&text).unwrap(), now);
}
