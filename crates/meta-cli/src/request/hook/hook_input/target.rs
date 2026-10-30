use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::shell_command::ShellCommand;
use crate::request::hook::HookEvent;
use crate::request::hook::reply::{Denial, Reply, Verdict};
use crate::request::project::{Config, Project};
use crate::request::repository::Repository;

use super::HookInput;

#[derive(Debug, Default)]
pub struct Target {
    projects: Vec<Project>,
    unresolved: Vec<OsString>,
}

impl Target {
    const REPOSITORY_FLAGS: &'static [&'static str] = &["-R", "--repo"];
    const JOINED_FLAG: &'static str = "--repo=";
    const FLAG_PAIR_LEN: usize = 2;
    const STRICTEST: &'static str = "protected = [\"main\", \"master\"]\n";

    pub fn resolved_from(command: &ShellCommand, cwd: &Path) -> Self {
        let looked: Vec<Result<Option<Project>, PathBuf>> =
            command.named_directories(cwd).filter(|directory| directory != cwd).map(Self::looked_up).collect();
        let (found, lost): (Vec<_>, Vec<_>) = looked.into_iter().partition(Result::is_ok);
        let projects: Vec<Project> = found
            .into_iter()
            .flatten()
            .flatten()
            .map(|project| (project.root().to_path_buf(), project))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect();
        let unmatched = Self::repositories(command).into_iter().filter(|name| Self::owned(&projects, name).is_none());
        let lost = lost.into_iter().filter_map(Result::err).map(PathBuf::into_os_string);
        let unresolved = lost.chain(unmatched.map(OsString::from)).collect();
        Self { projects, unresolved }
    }

    fn looked_up(directory: PathBuf) -> Result<Option<Project>, PathBuf> {
        if directory.exists() { Project::found(&directory).map_err(|_| directory.clone()) } else { Err(directory) }
    }

    fn owned(projects: &[Project], name: &str) -> Option<()> {
        let named = name.parse::<Repository>().ok()?;
        projects.iter().find(|project| project.repository().ok().as_ref() == Some(&named)).map(|_| ())
    }

    pub fn denial(&self, event: HookEvent, command: &ShellCommand, input: &HookInput) -> Option<Reply> {
        let judged =
            self.projects.iter().map(|project| event.reply(project, input, event.budget(project.ci_settings())));
        let strictest: Config = toml::from_str(Self::STRICTEST).unwrap_or_default();
        let strict = Some(&self.unresolved)
            .filter(|unresolved| !unresolved.is_empty())
            .and_then(|_| command.refusal(&strictest, None))
            .map(|why| Reply::new(event, Verdict::Deny(Denial::from(why)), None));
        judged.chain(strict).find(Reply::denied)
    }

    fn repositories(command: &ShellCommand) -> Vec<String> {
        let flagged = |window: &[String]| match window {
            [flag, name] if Self::REPOSITORY_FLAGS.contains(&flag.as_str()) => Some(name.clone()),
            _ => None,
        };
        let joined = |word: &String| word.strip_prefix(Self::JOINED_FLAG).map(str::to_owned);
        command
            .segments()
            .flat_map(|words| {
                words.windows(Self::FLAG_PAIR_LEN).filter_map(flagged).chain(words.iter().filter_map(joined))
            })
            .collect()
    }
}
