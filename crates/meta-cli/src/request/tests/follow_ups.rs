use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::request::local::LocalClone;
use crate::request::project::{CiSettings, Config};
use crate::request::{Call, Environment, FollowUp};

const HOME: &str = "/home/agent";
const WATCH: [&str; 3] = ["ci-watch", "SESSION", "abc123"];
const ROOT: &str = "/work/clone";
const TRACE_SWITCH: (&str, &str) = ("META_TRACE", "off");

fn variables() -> BTreeMap<OsString, OsString> {
    [("HOME", HOME), TRACE_SWITCH].map(|(name, value)| (OsString::from(name), OsString::from(value))).into()
}

#[test]
fn an_answer_hands_back_the_follow_ups_its_call_queued_as_calls_in_their_directory_and_the_environment_keeps_home() {
    let environment = Environment::new(PathBuf::from("/"), PathBuf::from("meta"), variables());
    assert_eq!(environment.home(), Some(PathBuf::from(HOME)));
    environment.then(FollowUp::new(WATCH.map(String::from).to_vec(), PathBuf::from(ROOT)));
    let call = Call::new(vec!["meta".to_owned(), "bogus".to_owned()], environment, Box::new(std::io::empty()));
    let followed = call.answered().then();
    assert_eq!(followed.len(), 1);
    let shown = format!("{:?}", followed[0]);
    assert!(shown.starts_with("Call {") && shown.contains("ci-watch") && shown.contains(ROOT), "{shown}");
    assert!(shown.contains("abc123") && shown.contains(HOME), "{shown}");
}

#[test]
fn a_clone_in_an_environment_reports_that_environment() {
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let environment = Environment::new(PathBuf::from("/"), PathBuf::from("meta"), variables());
    let clone = LocalClone::new(PathBuf::from(ROOT), PathBuf::from(ROOT).join(".git"), settings);
    assert_eq!(clone.environment().home(), None);
    let clone = clone.in_environment(environment);
    assert_eq!(clone.environment().home(), Some(PathBuf::from(HOME)));
}
