use std::path::Path;
use std::process::Command;

pub enum Skeleton {
    Bare,
    Working,
}

impl Skeleton {
    const TEMPLATE: &'static str = concat!("--template=", env!("CARGO_MANIFEST_DIR"), "/tests/support/template");

    pub fn lay(self, dir: &Path, branch: &str) {
        let shape = match self {
            Self::Bare => "--bare",
            Self::Working => "--quiet",
        };
        let init = Command::new("git").args(["init", "--quiet", shape, Self::TEMPLATE, "-b", branch]).arg(dir).status();
        assert!(init.unwrap().success());
    }
}
