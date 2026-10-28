use std::ffi::OsString;
use std::path::PathBuf;

use super::{Beat, SETTINGS};
use crate::request::Request;
use crate::request::heartbeat::{Heartbeat, LaunchdPlist};
use crate::request::local::LocalClone;
use crate::request::project::CiSettings;

const CHECKOUT: &str = "/work/a&b";
const GIT_DIR: &str = "/work/a&b/.git";
const PROGRAM: &str = "/bin/meta";
const SEARCH_PATH: &str = "/usr/bin";

#[test]
fn the_plist_runs_tick_in_the_clone_on_the_policy_interval_and_escapes_markup() {
    let clone = LocalClone::new(PathBuf::from(CHECKOUT), PathBuf::from(GIT_DIR), CiSettings::default());
    let settings = Beat::settings(SETTINGS).unwrap();
    let plist = LaunchdPlist::new(PathBuf::from(PROGRAM), &clone, &settings, OsString::from(SEARCH_PATH));
    assert_eq!(plist.to_string(), include_str!("plist.plist"));
}

#[test]
fn heartbeat_names_parse_and_anything_else_is_refused() {
    let parse = |word: &str| Request::try_from(["meta", "heartbeat", word].map(String::from).to_vec());
    assert!(matches!(parse("tick"), Ok(Request::Heartbeat(Heartbeat::Tick))));
    assert!(matches!(parse("flags"), Ok(Request::Heartbeat(Heartbeat::Flags))));
    assert!(matches!(parse("plist"), Ok(Request::Heartbeat(Heartbeat::Plist))));
    assert!(parse("install").is_err());
}
