mod reason;

use std::str::FromStr;

use serde::Deserialize;

use super::message::Message;
use reason::Reason;

#[derive(Debug, Deserialize)]
pub(super) struct CargoLine {
    reason: Reason,
    message: Option<Message>,
}

impl CargoLine {
    pub(super) fn compiler_message(self) -> Option<Message> {
        self.message.filter(|_| self.reason == Reason::CompilerMessage)
    }
}

impl FromStr for CargoLine {
    type Err = serde_json::Error;

    fn from_str(line: &str) -> Result<CargoLine, serde_json::Error> {
        serde_json::from_str(line)
    }
}
