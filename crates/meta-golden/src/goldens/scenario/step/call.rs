mod tool_call;

use meta_run::{EnvPolicy, GhArgv, GitArgv, Port, Run};
use serde::Deserialize;

use crate::places::Places;
use tool_call::ToolCall;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Call {
    Run(Vec<String>),
    Git(ToolCall<GitArgv>),
    Gh(ToolCall<GhArgv>),
}

impl Call {
    pub fn run(&self, places: &Places) -> Run {
        match self {
            Self::Run(words) => {
                let words: Vec<_> = words.iter().map(|word| places.shown(word)).collect();
                let run = |(program, operands): (_, &[_])| Run::new(program, EnvPolicy::Inherit).args(operands);
                words.split_first().map_or_else(|| Run::new("", EnvPolicy::Inherit), run).dir(places.root())
            }
            Self::Git(call) => call.run(&Port::new(places.root()), places),
            Self::Gh(call) => call.run(&Port::new(places.root()), places),
        }
    }
}
