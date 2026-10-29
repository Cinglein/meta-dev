use std::str::FromStr;

use serde::Deserialize;

use super::message::Message;

#[derive(Debug, Deserialize)]
pub(super) struct CargoLine {
    message: Option<Message>,
}

impl CargoLine {
    pub(super) fn error(self) -> Option<Message> {
        self.message.filter(Message::is_error)
    }
}

impl FromStr for CargoLine {
    type Err = serde_json::Error;

    fn from_str(line: &str) -> Result<CargoLine, serde_json::Error> {
        serde_json::from_str(line)
    }
}
