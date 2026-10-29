use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use task::{Policy, Site, Workspace};

use crate::task::failure::Failure;

pub struct DenyConfig(PathBuf);

impl DenyConfig {
    const PATH: &str = "meta/deny.toml";

    pub fn written(workspace: &Workspace, policy: &Policy) -> Result<DenyConfig, Failure> {
        let path = workspace.target().path().join(Self::PATH);
        let io = |error| Failure::from(task::Failure::Io { site: Site::File(path.clone()), error });
        path.parent().map_or(Ok(()), fs::create_dir_all).map_err(io)?;
        File::create(&path).and_then(|mut file| write!(file, "{}", policy.supply_chain())).map_err(io)?;
        Ok(DenyConfig(path))
    }
}

impl fmt::Display for DenyConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}

#[cfg(test)]
mod tests {
    use meta_golden::Golden;
    use settings::EmbeddedDefaults;
    use task::Policy;

    use super::DenyConfig;

    const PATH: &str = "target/meta/deny.toml";
    const SEMVER: [&str; 1] = ["meta-xtask"];

    #[test]
    fn the_policy_writes_a_deny_config_that_denies_unknown_sources_and_unlisted_licences() {
        let policy = Policy::defaults();
        Golden::for_test(module_path!(), "deny").checked(&policy.supply_chain().to_string()).unwrap();
        assert_eq!(DenyConfig(PATH.into()).to_string(), PATH);
        assert!(policy.supply_chain().semver().iter().map(ToString::to_string).eq(SEMVER));
    }
}
