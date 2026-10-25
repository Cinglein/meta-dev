use super::CliFailure;
use super::failure_kind::FailureKind;
use super::message::Message;

#[derive(Debug)]
pub struct Invalid;

impl FailureKind for Invalid {
    fn failure(message: Message) -> CliFailure {
        CliFailure::Invalid(message)
    }
}
