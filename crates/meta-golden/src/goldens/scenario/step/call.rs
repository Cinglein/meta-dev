mod tool_call;

use meta_run::{GhArgv, GitArgv, MetaArgv, Port, Run, Tool};
use serde::Deserialize;

use crate::goldens::Reach;
use crate::places::Places;
use tool_call::ToolCall;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Call {
    Git(ToolCall<GitArgv>),
    Gh(ToolCall<GhArgv>),
    Meta(ToolCall<MetaArgv>),
}

impl Call {
    pub fn reach(&self) -> Reach {
        match self {
            Self::Gh(_) => Reach::Online,
            Self::Git(_) | Self::Meta(_) => Reach::Offline,
        }
    }

    pub fn run(&self, places: &Places) -> Run {
        match self {
            Self::Git(call) => call.run(&Port::new(places.root()), places),
            Self::Gh(call) => call.run(&Port::new(places.root()), places),
            Self::Meta(call) => call.run(&Port::named(places.program(MetaArgv::PROGRAM), places.root()), places),
        }
    }
}
