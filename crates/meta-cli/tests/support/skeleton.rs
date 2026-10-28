use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use meta_door::Door;

pub struct Skeleton;

impl Skeleton {
    const TEMPLATE: &'static str = concat!("--template=", env!("CARGO_MANIFEST_DIR"), "/tests/support/template");
    const BARE: [&'static str; 6] = ["init", "--quiet", "--bare", Self::TEMPLATE, "-b", "main"];
    const INIT: [&'static str; 2] = ["init", "--quiet"];
    const TEMPLATED: &'static str = "--template=";
    const BRANCH: &'static str = "-b";
    const CONFIG: &'static str = "config";
    const REMOTE: &'static str = "[maintenance]\n\tauto = false\n[remote \"origin\"]\n\turl = ";
    const FETCH: &'static str = "\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n";

    pub fn laid(origin: &Path, clone: &Path, branch: &str) {
        let bare = Command::new("git").args(Self::BARE).arg(origin).status();
        let template = tempfile::tempdir().unwrap();
        let config = [Self::REMOTE, origin.to_str().unwrap(), Self::FETCH].concat();
        Door::write(template.path().join(Self::CONFIG), config).unwrap();
        let mut templated = OsString::from(Self::TEMPLATED);
        templated.push(template.path());
        let init = Command::new("git").args(Self::INIT).arg(templated).args([Self::BRANCH, branch]).arg(clone).status();
        assert!(bare.unwrap().success() && init.unwrap().success());
    }
}
