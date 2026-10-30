mod known_projects;
mod unleased;

use std::path::PathBuf;

use meta_log::GitLog;

use crate::request::dispatch::Prompt;
use crate::request::hook::HookEvent;
use crate::request::output::Listed;
use crate::request::project::Project;
use known_projects::KnownProjects;
use unleased::Unleased;

#[derive(Debug)]
pub struct AgentCall<'input> {
    cwd: PathBuf,
    prompt: Option<&'input Prompt>,
    nested: Option<()>,
}

impl<'input> AgentCall<'input> {
    pub fn new(cwd: PathBuf, prompt: Option<&'input Prompt>, nested: Option<()>) -> Self {
        Self { cwd, prompt, nested }
    }

    pub fn refusal(&self, here: Option<&Project>) -> Option<Listed> {
        let prompt = self.nested.map_or(Ok(self.prompt), |()| Err(Unleased::Nested));
        let prompt = prompt.and_then(|prompt| prompt.ok_or(Unleased::NoPrompt));
        let admitted =
            prompt.and_then(|prompt| prompt.admitted(&self.authority(here, prompt)?).map_err(Unleased::Refused));
        admitted.err().map(Listed::new)
    }

    fn authority(&self, here: Option<&Project>, prompt: &Prompt) -> Result<GitLog, Unleased> {
        let log = |project: &Project| {
            let budget = HookEvent::PreToolUse.budget(project.ci_settings());
            project.log(budget).map_err(|failure| Unleased::Refused(Listed::new(failure)))
        };
        match (here, prompt.project()) {
            (Some(project), None) => log(project),
            (Some(project), Some(named)) => {
                let roots = named.canonicalize().ok().zip(project.root().canonicalize().ok());
                roots.filter(|(named, root)| named == root).ok_or(Unleased::Elsewhere(named)).and_then(|_| log(project))
            }
            (None, Some(named)) => Self::named(named).and_then(|project| log(&project)),
            (None, None) => Err(Unleased::NoProject(self.cwd.clone())),
        }
    }

    fn named(named: PathBuf) -> Result<Project, Unleased> {
        let absolute = Some(named.clone()).filter(|path| path.is_absolute()).ok_or(Unleased::Relative(named))?;
        KnownProjects::read(&KnownProjects::file())?
            .admits(&absolute)
            .ok_or_else(|| Unleased::Unknown(absolute.clone()))?;
        let found = Project::found(&absolute).map_err(|failure| Unleased::Refused(Listed::new(failure)))?;
        found.ok_or(Unleased::NoProject(absolute))
    }
}
