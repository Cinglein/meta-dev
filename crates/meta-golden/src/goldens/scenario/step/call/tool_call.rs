use std::fmt::Display;
use std::str::FromStr;

use meta_run::{Port, Run, Tool};
use serde::Deserialize;

use crate::places::Places;

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "Argv: FromStr<Err: Display>"))]
pub struct ToolCall<Argv> {
    #[serde(deserialize_with = "ToolCall::<Argv>::named")]
    call: Argv,
    #[serde(default)]
    operands: Vec<String>,
}

impl<Argv: Tool> ToolCall<Argv> {
    pub fn run(&self, port: &Port<Argv>, places: &Places) -> Run {
        port.run(self.call, self.operands.iter().map(|operand| places.shown(operand)))
    }
}

impl<Argv: FromStr<Err: Display>> ToolCall<Argv> {
    fn named<'de, Words: serde::Deserializer<'de>>(words: Words) -> Result<Argv, Words::Error> {
        String::deserialize(words)?.parse().map_err(serde::de::Error::custom)
    }
}
