use std::ffi::OsString;
use std::path::PathBuf;

use meta_golden::Golden;

use super::{Beat, SETTINGS};
use crate::request::Request;
use crate::request::heartbeat::LaunchdPlist;
use crate::request::local::LocalClone;
use crate::request::project::{CiSettings, Config};

const CHECKOUT: &str = "/work/a&b";
const GIT_DIR: &str = "/work/a&b/.git";
const PROGRAM: &str = "/bin/meta";
const SEARCH_PATH: &str = "/usr/bin";

#[test]
fn the_plist_runs_tick_in_the_clone_on_the_policy_interval_and_escapes_markup() {
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let clone = LocalClone::new(PathBuf::from(CHECKOUT), PathBuf::from(GIT_DIR), settings);
    let settings = Beat::settings(SETTINGS).unwrap();
    let plist = LaunchdPlist::new(PathBuf::from(PROGRAM), &clone, &settings, OsString::from(SEARCH_PATH));
    Golden::for_test(module_path!(), "plist").shown(&plist).unwrap();
}

#[test]
fn heartbeat_names_parse_and_anything_else_is_refused() {
    let parse = |word: &str| Request::try_from(["meta", "heartbeat", word].map(String::from).to_vec());
    let parsed = ["tick", "flags", "plist", "install"].map(parse);
    Golden::for_test(module_path!(), "parsed").debugged(&parsed).unwrap();
}
