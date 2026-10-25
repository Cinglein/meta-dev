use std::env;
use std::fs;
use std::path::PathBuf;

use meta_ci::CiConfig;
use meta_lints::{LintConfig, Workspace};
use serde::Deserialize;

use crate::task::failure::Failure;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    lints: LintConfig,
    ci: CiConfig,
}

#[derive(Clone, Debug)]
pub struct Project {
    root: PathBuf,
    settings: Settings,
}

impl Project {
    const CONFIG: &str = "xtask.toml";

    pub fn locate() -> Result<Project, Failure> {
        let start = env::current_dir()?;
        let root = start
            .ancestors()
            .find(|directory| directory.join(Self::CONFIG).is_file())
            .ok_or(Failure::Unconfigured(Self::CONFIG))?
            .to_path_buf();
        let settings = toml::from_str(&fs::read_to_string(root.join(Self::CONFIG))?)?;
        Ok(Project { root, settings })
    }

    pub fn lints(&self) -> Workspace {
        Workspace::new(self.root.clone(), self.settings.lints.clone())
    }

    pub fn run_ci(&self) -> Result<(), meta_ci::Failure> {
        self.settings.ci.run_in(&self.root)
    }
}
