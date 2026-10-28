use std::ffi::OsStr;

use super::git::Git;
use crate::request::cli_failure::CliFailure;

pub struct WorktreeIdentity<'text> {
    name: &'text str,
    email: &'text str,
}

impl<'text> WorktreeIdentity<'text> {
    const SET: [&'static str; 2] = ["config", "--worktree"];
    const KEYS: [[&'static str; 2]; 3] =
        [["user.name", "user.email"], ["author.name", "author.email"], ["committer.name", "committer.email"]];

    pub fn new(name: &'text str, email: &'text str) -> Self {
        Self { name, email }
    }

    pub fn configured(&self, worktree: &Git<'_>) -> Result<(), CliFailure> {
        let [config, scope] = Self::SET;
        Self::KEYS
            .iter()
            .flat_map(|[name, email]| [[*name, self.name], [*email, self.email]])
            .try_for_each(|[key, value]| worktree.output(&[config, scope, key, value].map(OsStr::new)).map(drop))
    }
}
