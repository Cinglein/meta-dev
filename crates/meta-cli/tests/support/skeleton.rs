use std::path::Path;
use std::process::{Command, Stdio};

pub struct Skeleton;

impl Skeleton {
    const TEMPLATE: &'static str = concat!("--template=", env!("CARGO_MANIFEST_DIR"), "/tests/support/template");
    const BARE: [&'static str; 6] = ["init", "--quiet", "--bare", Self::TEMPLATE, "-b", "main"];
    const CLONE: [&'static str; 3] = ["clone", "--quiet", Self::TEMPLATE];

    pub fn laid(origin: &Path, clone: &Path) {
        let bare = Command::new("git").args(Self::BARE).arg(origin).status();
        let cloned = Command::new("git").args(Self::CLONE).arg(origin).arg(clone).stderr(Stdio::null()).status();
        assert!(bare.unwrap().success() && cloned.unwrap().success());
    }
}
