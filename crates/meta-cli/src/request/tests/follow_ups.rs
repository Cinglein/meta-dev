use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use meta_golden::Golden;

use crate::request::local::LocalClone;
use crate::request::project::{CiSettings, Config};
use crate::request::{Call, Environment, FollowUp};

const HOME: &str = "/home/agent";
const WATCH: [&str; 3] = ["ci-watch", "SESSION", "abc123"];
const ROOT: &str = "/work/clone";
const VARIABLES: [(&str, &str); 2] = [("HOME", HOME), ("META_TRACE", "off")];

struct Setup;

impl Setup {
    fn environment() -> Environment {
        let variables: BTreeMap<OsString, OsString> =
            VARIABLES.map(|(name, value)| (OsString::from(name), OsString::from(value))).into();
        Environment::new(PathBuf::from("/"), PathBuf::from("meta"), variables)
    }
}

#[test]
fn an_answer_hands_back_the_follow_ups_its_call_queued_as_calls_in_their_directory() {
    let environment = Setup::environment();
    assert_eq!(environment.home(), Some(PathBuf::from(HOME)));
    environment.then(FollowUp::new(WATCH.map(String::from).to_vec(), PathBuf::from(ROOT)));
    let call = Call::new(vec!["meta".to_owned(), "bogus".to_owned()], environment, Box::new(std::io::empty()));
    Golden::for_test(module_path!(), "followed").debugged(&call.answered().then()).unwrap();
}

#[test]
fn a_clone_in_an_environment_reports_that_environment() {
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let clone = LocalClone::new(PathBuf::from(ROOT), PathBuf::from(ROOT).join(".git"), settings);
    assert_eq!(clone.environment().home(), None);
    let clone = clone.in_environment(Setup::environment());
    assert_eq!(clone.environment().home(), Some(PathBuf::from(HOME)));
}
