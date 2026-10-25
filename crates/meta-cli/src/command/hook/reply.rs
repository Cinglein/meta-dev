use std::fmt;

use crate::command::output::Listed;

#[derive(Debug, Default)]
pub enum Reply {
    #[default]
    Silent,
    SessionContext(Listed),
    Deny(Listed),
    ToolContext(Listed),
}

impl fmt::Display for Reply {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silent => Ok(()),
            Self::SessionContext(text) => write!(formatter, "{text}"),
            Self::Deny(reason) => write!(
                formatter,
                "{}",
                serde_json::json!({ "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": format!("meta-development: {reason}"),
                }})
            ),
            Self::ToolContext(text) => write!(
                formatter,
                "{}",
                serde_json::json!({ "hookSpecificOutput": {
                    "hookEventName": "PostToolUse",
                    "additionalContext": format!("meta-development: {text}"),
                }})
            ),
        }
    }
}
