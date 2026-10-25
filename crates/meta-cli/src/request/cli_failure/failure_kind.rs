use super::CliFailure;
use super::message::Message;

pub trait FailureKind {
    fn failure(message: Message) -> CliFailure;
}
