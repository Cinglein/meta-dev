mod prior_lints;
mod prior_settings;
mod settings;
mod surfaces;

use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::prior::{Prior, Revision};
use crate::site::Site;
use crate::workspace::Workspace;
use prior_lints::PriorLints;
use prior_settings::PriorSettings;
use settings::Settings;
use surfaces::Changed;

#[derive(Clone, Debug)]
pub struct Project {
    root: PathBuf,
    settings: Settings,
}

impl Project {
    const CONFIG: &str = "xtask.toml";

    pub fn locate(start: &Path) -> Result<Project, Failure> {
        let root = start
            .ancestors()
            .find(|directory| directory.join(Self::CONFIG).is_file())
            .ok_or(Failure::Unconfigured(Self::CONFIG))?
            .to_path_buf();
        let path = root.join(Self::CONFIG);
        let text = fs::read_to_string(&path).map_err(|error| Failure::Io { site: Site::of_path(&path), error })?;
        let settings = toml::from_str(&text)
            .map_err(|error| Failure::Config { site: Site::of_path(&path), error: Box::new(error) })?;
        Ok(Project { root, settings })
    }

    #[must_use]
    pub fn lints(&self) -> Workspace {
        Workspace::new(self.root.clone(), self.settings.lints().clone())
    }

    pub fn prior(&self) -> Result<Prior, Failure> {
        let revision = Revision::merge_base(&self.root)?;
        PriorSettings::read_at(&revision, Path::new(Self::CONFIG))?
            .map_or_else(|| PriorLints::new(self.settings.lints().baseline().to_path_buf()), PriorSettings::lints)
            .prior(&revision)
    }

    pub fn changed_surfaces(&self) -> Result<Changed, Failure> {
        let revision = Revision::merge_base(&self.root)?;
        let prior = PriorSettings::read_at(&revision, Path::new(Self::CONFIG))?
            .map(PriorSettings::surfaces)
            .unwrap_or_default();
        self.settings.surfaces().union(prior).changed(&revision)
    }
}
