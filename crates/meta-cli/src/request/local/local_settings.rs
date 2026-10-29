mod identity;
mod monitor_settings;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

use super::local_file::LocalFile;
use crate::request::cli_failure::{CliFailure, Setup};
pub use identity::Identity;
pub use monitor_settings::{AgentRoot, Cause, MonitorSettings, SparedName, TickCount};

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
    const SIBLING_SUFFIX: &'static str = "-worktrees.noindex";
    const LEGACY_SUFFIX: &'static str = "-worktrees";
    const HOME: &'static str = "HOME";

    pub fn read(checkout: &Path) -> Result<Self, CliFailure> {
        let file = LocalFile::new(checkout.join(Self::FILE));
        file.text()?.parse().map_err(|failure: toml::de::Error| file.failure(failure))
    }

    pub fn admitted(present: Option<()>, ignored: impl FnOnce() -> Option<()>) -> Result<(), CliFailure> {
        present.map_or(Some(()), |()| ignored()).ok_or_else(|| {
            CliFailure::raised::<Setup>(format!(
                "{file} is not gitignored; add /{file} to .gitignore",
                file = Self::FILE
            ))
        })
    }

    pub fn worktree_root(&self, checkout: &Path) -> PathBuf {
        self.worktree_root
            .as_ref()
            .map_or_else(|| Self::sibling(checkout, Self::SIBLING_SUFFIX), |root| checkout.join(root))
    }

    pub fn worktree_roots(&self, checkout: &Path) -> Vec<PathBuf> {
        let legacy = self.worktree_root.is_none().then(|| Self::sibling(checkout, Self::LEGACY_SUFFIX));
        [self.worktree_root(checkout)].into_iter().chain(legacy).collect()
    }

    fn sibling(checkout: &Path, suffix: &str) -> PathBuf {
        let name = checkout.file_name().unwrap_or_default().to_string_lossy();
        checkout.with_file_name(format!("{name}{suffix}"))
    }

    pub fn monitor(&self, checkout: &Path) -> Result<MonitorSettings, CliFailure> {
        let home = std::env::var_os(Self::HOME)
            .map(PathBuf::from)
            .ok_or_else(|| CliFailure::raised::<Setup>("monitor: $HOME is unset"))?;
        MonitorSettings::merged(self.monitor.clone(), checkout, &self.worktree_roots(checkout), &home)
            .map_err(|failure| CliFailure::raised::<Setup>(format!("{} [monitor]: {failure}", Self::FILE)))
    }

    pub fn identity(&self, name: Option<&str>, email: Option<&str>) -> Result<Identity, CliFailure> {
        let chosen = self.identity.clone().map_or_else(|| Identity::from_git(name, email), Ok);
        chosen.map_err(|refused| CliFailure::raised::<Setup>(format!("--worktree: {refused}")))
    }
}

impl FromStr for LocalSettings {
    type Err = toml::de::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        toml::from_str(text)
    }
}
