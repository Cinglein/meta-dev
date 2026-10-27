use meta_severity::Scales;

use super::{DIVISION, Meta, OWNER, PROGRAM};
use crate::request::Request;
use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::scale_policy::ScalePolicy;

const OVERRIDDEN: &str =
    r#"{"features-and-maintenance": [{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]}"#;
const BLOCKER: &str = "blocker";
const NUISANCE: &str = "nuisance";
const RAISED_FROM_MAJOR: &str = "blocker (raised from major";

impl Meta {
    pub(super) fn under(&self, scales: &Scales, words: &[&str]) -> Result<String, CliFailure> {
        let args = std::iter::once(PROGRAM).chain(words.iter().copied()).map(String::from).collect::<Vec<_>>();
        let actor = || OWNER.parse().map_err(CliFailure::raised::<Invalid>);
        Request::try_from(args)
            .and_then(|request| request.answer_from(&self.0, &ScalePolicy::from(scales.clone()), actor))
            .map(|output| output.to_string())
    }
}

#[test]
fn writes_go_through_while_the_policy_strands_a_rated_item_and_it_can_be_rated_again() {
    let meta = Meta::default();
    let concern = meta.written(OWNER, &[&RATED_CONCERN[..], &[MAJOR]].concat());
    let overridden: Scales = serde_json::from_str(OVERRIDDEN).unwrap();
    let written = meta.under(&overridden, &[&RATED_ISSUE[..], &[NUISANCE]].concat()).unwrap();
    let rated = meta.under(&overridden, &[&[RATE, concern.as_str(), BLOCKER][..], &SCENARIO[..]].concat()).unwrap();
    assert!(written.contains(NUISANCE) && rated.contains(RAISED_FROM_MAJOR));
}

const RATED_CONCERN: [&str; 6] = ["concern", "map", "lost", "--division", DIVISION, "--severity"];
const RATED_ISSUE: [&str; 5] = ["issue", "crash", "--division", DIVISION, "--severity"];
const RATE: &str = "rate";
const MAJOR: &str = "major";
const MINOR: &str = "minor";
const CRITICAL: &str = "critical";
const SCENARIO: [&str; 2] = ["--scenario", "offline start shows no map"];
const NEEDS_SCENARIO: &str = "raising from minor needs --scenario";
const SHOWN: &str = "critical (raised from minor: offline start shows no map)";
const UNKNOWN: &str = "blocker is not a level";

#[test]
fn a_rated_concern_is_lowered_freely_and_raised_only_with_a_scenario() {
    let meta = Meta::default();
    let concern = meta.written(OWNER, &[&RATED_CONCERN[..], &[MAJOR]].concat());
    meta.run(OWNER, &[RATE, &concern, MINOR]).unwrap();
    let refused = meta.run(OWNER, &[RATE, &concern, CRITICAL]).unwrap_err().to_string();
    let raised = meta.run(OWNER, &[&[RATE, concern.as_str(), CRITICAL][..], &SCENARIO[..]].concat()).unwrap();
    assert!(refused.contains(NEEDS_SCENARIO) && raised.contains(SHOWN));
    assert!(meta.run(OWNER, &[RATE, &concern]).is_err() && meta.run(OWNER, &[RATE, &concern, CRITICAL]).is_err());
}

#[test]
fn a_new_item_takes_only_a_level_of_its_divisions_scale() {
    let meta = Meta::default();
    let issue = meta.run(OWNER, &[&RATED_ISSUE[..], &[MINOR]].concat()).unwrap();
    let refused = meta.run(OWNER, &[&RATED_ISSUE[..], &["blocker"]].concat()).unwrap_err().to_string();
    assert!(issue.contains(" !minor") && refused.contains(UNKNOWN));
}
