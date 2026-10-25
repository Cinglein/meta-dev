use super::CliFailure;
use super::failure_kind::FailureKind;
use super::message::Message;

#[derive(Debug)]
pub struct Misuse;

impl FailureKind for Misuse {
    fn failure(message: Message) -> CliFailure {
        CliFailure::Usage(message)
    }
}
