mod identity;
mod monitor_settings;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

use super::local_file::LocalFile;
use crate::request::cli_failure::{CliFailure, Setup};
use identity::Identity;
pub use monitor_settings::{Cause, CoreShare, MonitorSettings, SparedName};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalSettings {
    #[serde(default)]
    worktree_root: Option<PathBuf>,
    #[serde(default)]
    identity: Option<Identity>,
    #[serde(default)]
    monitor: toml::Table,
}

impl LocalSettings {
    pub const FILE: &'static str = "meta.local.toml";
    const SIBLING_SUFFIX: &'static str = "-worktrees";

    pub fn read(checkout: &Path) -> Result<Self, CliFailure> {
        let file = LocalFile::new(checkout.join(Self::FILE));
        file.text()?.parse().map_err(|failure: toml::de::Error| file.failure(failure))
    }

    pub fn worktree_root(&self, checkout: &Path) -> PathBuf {
        self.worktree_root.as_ref().map_or_else(
            || {
                let name = checkout.file_name().unwrap_or_default().to_string_lossy();
                checkout.with_file_name(format!("{name}{}", Self::SIBLING_SUFFIX))
            },
            |root| checkout.join(root),
        )
    }

    pub fn monitor(&self, checkout: &Path) -> Result<MonitorSettings, CliFailure> {
        MonitorSettings::merged(self.monitor.clone(), checkout)
            .map_err(|failure| CliFailure::raised::<Setup>(format!("{} [monitor]: {failure}", Self::FILE)))
    }

    pub fn identity(&self) -> Result<&Identity, CliFailure> {
        self.identity.as_ref().ok_or_else(|| {
            CliFailure::raised::<Setup>(format!(
                "{}: --worktree needs [identity] name and email (a GitHub noreply address)",
                Self::FILE
            ))
        })
    }
}

impl FromStr for LocalSettings {
    type Err = toml::de::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        toml::from_str(text)
    }
}
