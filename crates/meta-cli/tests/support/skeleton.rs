use std::path::Path;

use meta_door::{Door, Layout};

pub struct Skeleton;

impl Skeleton {
    const ENTRY: &'static str = ".git";
    const MAIN: &'static str = "main";
    const MAINTENANCE: &'static str = "[maintenance]\n\tauto = false\n";
    const REMOTE: &'static str = "[remote \"origin\"]\n\turl = ";
    const FETCH: &'static str = "\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n";

    pub fn laid(origin: &Path, clone: &Path, branch: &str) {
        let remote = [Self::MAINTENANCE, Self::REMOTE, origin.to_str().unwrap(), Self::FETCH].concat();
        Door::skeleton(origin, Layout::Bare, Self::MAIN, Self::MAINTENANCE).unwrap();
        Door::skeleton(&clone.join(Self::ENTRY), Layout::Working, branch, &remote).unwrap();
    }
}
