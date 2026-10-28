use super::status::Status;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit<'recording> {
    status: Status,
    err: &'recording str,
}

impl<'recording> Exit<'recording> {
    pub(crate) fn new(status: Status, err: &'recording str) -> Self {
        Self { status, err }
    }
}
