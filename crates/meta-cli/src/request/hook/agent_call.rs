mod agent_input;
mod tool_call;
mod unleased;

use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::HookEvent;
use super::reply::{Denial, Verdict};
use crate::request::dispatch::Prompt;
use crate::request::output::Listed;
use crate::request::project::Project;
use agent_input::AgentInput;
use tool_call::ToolCall;
use unleased::Unleased;

#[derive(Debug, Default, Deserialize)]
pub struct AgentCall {
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    tool_input: Option<AgentInput>,
}

impl AgentCall {
    pub fn verdict(event: HookEvent, stdin: &str) -> Option<Verdict> {
        (event == HookEvent::PreToolUse).then_some(())?;
        ToolCall::agent(stdin)?;
        let call = serde_json::from_str::<Self>(stdin).map_err(|failure| Unleased::Unreadable(Listed::new(failure)));
        let admitted = call.and_then(|call| call.admitted(event));
        Some(admitted.map_or_else(|why| Verdict::Deny(Denial::from(Listed::new(why))), |()| Verdict::Allow))
    }

    fn admitted(&self, event: HookEvent) -> Result<(), Unleased> {
        let prompt = self.tool_input.as_ref().and_then(AgentInput::prompt).ok_or(Unleased::NoPrompt)?;
        let project = self.authority(prompt)?;
        let log = project.log(event.budget(project.ci_settings()));
        prompt.admitted(&log.map_err(|failure| Unleased::Refused(Listed::new(failure)))?).map_err(Unleased::Refused)
    }

    fn authority(&self, prompt: &Prompt) -> Result<Project, Unleased> {
        let here = self.cwd.clone().or_else(|| std::env::current_dir().ok()).unwrap_or_default();
        let found =
            |directory: &Path| Project::found(directory).map_err(|failure| Unleased::Refused(Listed::new(failure)));
        match (found(&here)?, prompt.project()) {
            (Some(project), None) => Ok(project),
            (Some(project), Some(named)) => {
                let roots = named.canonicalize().ok().zip(project.root().canonicalize().ok());
                roots.filter(|(named, root)| named == root).map(|_| project).ok_or(Unleased::Elsewhere(named))
            }
            (None, Some(named)) => {
                if named.is_absolute() {
                    found(&named)?.ok_or(Unleased::NoProject(named))
                } else {
                    Err(Unleased::Relative(named))
                }
            }
            (None, None) => Err(Unleased::NoProject(here)),
        }
    }
}
