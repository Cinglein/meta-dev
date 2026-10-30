mod agent_input;
mod tool_name;
mod unleased;

use std::path::PathBuf;

use serde::Deserialize;

use super::HookEvent;
use super::reply::{Reply, Verdict};
use crate::request::output::Listed;
use crate::request::project::Project;
use agent_input::AgentInput;
use tool_name::ToolName;
use unleased::Unleased;

#[derive(Debug, Default, Deserialize)]
pub struct AgentCall {
    #[serde(default)]
    tool_name: Option<ToolName>,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    tool_input: Option<AgentInput>,
}

impl AgentCall {
    pub fn reply(event: HookEvent, stdin: &str) -> Option<Reply> {
        let call = serde_json::from_str::<Self>(stdin).unwrap_or_default();
        (event == HookEvent::PreToolUse && call.tool_name == Some(ToolName::Agent)).then_some(())?;
        let refused = |why: Unleased| event.refused(Listed::new(why));
        Some(call.admitted(event).map_or_else(refused, |()| Reply::new(event, Verdict::Allow, None)))
    }

    fn admitted(&self, event: HookEvent) -> Result<(), Unleased> {
        let prompt = self.tool_input.as_ref().and_then(AgentInput::prompt).ok_or(Unleased::NoPrompt)?;
        let directory = prompt.project().or_else(|| self.cwd.clone()).unwrap_or_default();
        let found = Project::found(&directory).map_err(|failure| Unleased::Refused(Listed::new(failure)))?;
        let project = found.ok_or(Unleased::NoProject(directory))?;
        let log = project.log(event.budget(project.ci_settings()));
        prompt.admitted(&log.map_err(|failure| Unleased::Refused(Listed::new(failure)))?).map_err(Unleased::Refused)
    }
}
