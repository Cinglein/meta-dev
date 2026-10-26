use super::CliFailure;
use super::failure_kind::FailureKind;
use super::message::Message;

#[derive(Debug)]
pub struct Setup;

impl FailureKind for Setup {
    fn failure(message: Message) -> CliFailure {
        CliFailure::Setup(message)
    }
}
