mod unleased;

use std::path::PathBuf;

use meta_log::GitLog;

use crate::request::dispatch::Prompt;
use crate::request::hook::HookEvent;
use crate::request::output::Listed;
use crate::request::project::Project;
use unleased::Unleased;

#[derive(Debug)]
pub struct AgentCall<'input> {
    cwd: PathBuf,
    prompt: Option<&'input Prompt>,
}

impl<'input> AgentCall<'input> {
    pub fn new(cwd: PathBuf, prompt: Option<&'input Prompt>) -> Self {
        Self { cwd, prompt }
    }

    pub fn refusal(&self, here: Option<&Project>) -> Option<Listed> {
        let prompt = self.prompt.ok_or(Unleased::NoPrompt);
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
        let found = named.is_absolute().then(|| Project::found(&named));
        match found {
            Some(found) => {
                found.map_err(|failure| Unleased::Refused(Listed::new(failure)))?.ok_or(Unleased::NoProject(named))
            }
            None => Err(Unleased::Relative(named)),
        }
    }
}
