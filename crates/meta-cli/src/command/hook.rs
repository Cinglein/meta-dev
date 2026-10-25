mod hook_input;
mod reply;

use std::str::FromStr;

use meta_state::Scope;

use crate::command::Command;
use crate::command::cli_failure::CliFailure;
use crate::command::output::{Listed, Output};
use crate::command::project::Project;
use hook_input::HookInput;
use reply::Reply;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookEvent {
    SessionStart,
    PreToolUse,
    PostToolUse,
}

impl FromStr for HookEvent {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "session-start" => Ok(Self::SessionStart),
            "pre-tool-use" => Ok(Self::PreToolUse),
            "post-tool-use" => Ok(Self::PostToolUse),
            other => Err(CliFailure::usage(format!("unknown hook `{other}`"))),
        }
    }
}

impl HookEvent {
    pub fn respond(self) -> Result<Output, CliFailure> {
        let stdin = std::io::read_to_string(std::io::stdin()).map_err(CliFailure::invalid)?;
        let input: HookInput = serde_json::from_str(&stdin).map_err(CliFailure::invalid)?;
        let reply = Project::discover(&input.cwd()).map(|project| self.reply(&project, &input)).unwrap_or_default();
        Ok(Output::Listed(Listed::new(reply)))
    }

    fn reply(self, project: &Project, input: &HookInput) -> Reply {
        match self {
            Self::SessionStart => {
                let actor = input
                    .announced_actor()
                    .map_or_else(|| "$USER (META_ACTOR unset)".to_owned(), |actor| actor.to_string());
                let state = Command::List(None, Scope::Open)
                    .answer_in(project)
                    .map_or_else(|failure| format!("meta/log unreadable: {failure}\n"), |output| output.to_string());
                Reply::SessionContext(Listed::new(format!(
                    "meta-development state (meta/log). actor: {actor}. meta list | claim | done | release | issue | concern | open\n{state}"
                )))
            }
            Self::PreToolUse => {
                input.command().and_then(|command| command.refusal(project)).map_or(Reply::Silent, Reply::Deny)
            }
            Self::PostToolUse => input
                .command()
                .and_then(|command| command.commits())
                .and_then(|_| project.push_current_branch())
                .map_or(Reply::Silent, Reply::ToolContext),
        }
    }
}
