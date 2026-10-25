mod surfaces;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use meta_ci::CiConfig;
use meta_lints::{LintConfig, Prior, Revision, Workspace};
use serde::Deserialize;

use crate::task::failure::Failure;
use surfaces::{Changed, Surfaces};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    surfaces: Surfaces,
    lints: LintConfig,
    ci: CiConfig,
}

#[derive(Clone, Debug, Deserialize)]
struct PriorSettings {
    #[serde(default)]
    surfaces: Surfaces,
    lints: PriorLints,
}

#[derive(Clone, Debug, Deserialize)]
struct PriorLints {
    baseline: PathBuf,
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

    pub fn prior(&self) -> Result<Prior, Failure> {
        let revision = Revision::merge_base(&self.root)?;
        let settings = Self::prior_settings(&revision)?;
        let baseline =
            settings.map_or_else(|| self.settings.lints.baseline().to_path_buf(), |prior| prior.lints.baseline);
        Ok(Prior::read_at(&revision, &baseline)?)
    }

    pub fn changed_surfaces(&self) -> Result<Changed, Failure> {
        let revision = Revision::merge_base(&self.root)?;
        let prior = Self::prior_settings(&revision)?.map(|prior| prior.surfaces).unwrap_or_default();
        Ok(self.settings.surfaces.union(prior).changed(&revision)?)
    }

    fn prior_settings(revision: &Revision) -> Result<Option<PriorSettings>, Failure> {
        let prior = revision.file(Path::new(Self::CONFIG))?;
        Ok(prior.and_then(|text| toml::from_str::<PriorSettings>(&text).ok()))
    }

    pub fn run_ci(&self) -> Result<(), meta_ci::Failure> {
        self.settings.ci.run_in(&self.root)
    }
}
