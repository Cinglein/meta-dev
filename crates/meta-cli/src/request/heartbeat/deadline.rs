use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug)]
pub struct Deadline;

impl Deadline {
    pub fn within<Done: Send + 'static>(limit: Duration, work: impl FnOnce() -> Done + Send + 'static) -> Option<Done> {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || sender.send(work()).ok());
        receiver.recv_timeout(limit).ok()
    }
}
