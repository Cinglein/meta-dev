use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::HookInput;
use super::shell_command::{Pin, ShellCommand};
use crate::request::hook::HookEvent;
use crate::request::hook::reply::{Denial, Reply, Verdict};
use crate::request::project::{Config, Project};
use crate::request::repository::Repository;
use crate::request::scale_policy::ScalePolicy;

#[derive(Debug)]
pub struct Target {
    projects: Vec<Project>,
    unresolved: Vec<OsString>,
}

impl Target {
    const UNPINNED: &'static str = "unpinned";

    pub fn resolved_from(command: &ShellCommand, cwd: &Path, home: Option<&Project>) -> Self {
        let pins = command.pins(cwd);
        let directories: Vec<&PathBuf> = pins
            .iter()
            .filter_map(Pin::directory)
            .filter(|directory| directory.as_path() != cwd)
            .filter(|directory| home.is_none_or(|home| Project::located(directory).as_deref() != Some(home.root())))
            .collect();
        let projects: Vec<Project> = directories
            .iter()
            .filter(|directory| directory.exists())
            .filter_map(|directory| Project::found(directory).ok().flatten())
            .map(|project| (project.root().to_path_buf(), project))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect();
        let lost = directories
            .iter()
            .filter(|directory| !directory.exists() || Project::found(directory).is_err())
            .map(|directory| directory.as_os_str().to_owned());
        let known: Vec<&Project> = home.into_iter().chain(&projects).collect();
        let unmatched = pins.iter().filter_map(Pin::repository).filter(|name| Self::owned(&known, name).is_none());
        let unpinned = pins.iter().filter(|pin| **pin == Pin::Unpinned).map(|_| OsString::from(Self::UNPINNED));
        let unresolved = lost.chain(unmatched.map(OsString::from)).chain(unpinned).collect();
        Self { projects, unresolved }
    }

    pub fn denial(
        &self,
        event: HookEvent,
        command: &ShellCommand,
        input: &HookInput,
        home: Option<&Project>,
    ) -> Option<Reply> {
        let judged =
            self.projects.iter().map(|project| event.reply(project, input, event.budget(project.ci_settings())));
        let policy = home.and_then(|home| ScalePolicy::read(home.root()).ok()).unwrap_or_default();
        let lander = policy.queue().lander(input.announced_actor());
        let strict = Some(&self.unresolved)
            .filter(|unresolved| !unresolved.is_empty())
            .and_then(|_| command.refusal(Config::strictest(), lander.as_ref()))
            .map(|why| Reply::new(event, Verdict::Deny(Denial::from(why)), None));
        judged.chain(strict).find(Reply::denied)
    }

    fn owned(known: &[&Project], name: &str) -> Option<()> {
        let named = name.parse::<Repository>().ok()?;
        known.iter().find(|project| project.repository().ok().as_ref() == Some(&named)).map(|_| ())
    }
}
