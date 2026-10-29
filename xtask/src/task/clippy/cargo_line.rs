use serde::Deserialize;

use super::message::Message;

#[derive(Debug, Deserialize)]
pub(super) struct CargoLine {
    message: Option<Message>,
}

impl CargoLine {
    pub(super) fn message(self) -> Option<Message> {
        self.message
    }
}
