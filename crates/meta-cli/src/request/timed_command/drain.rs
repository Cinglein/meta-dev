use std::io::Read;
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

use super::process_group::ProcessGroup;

#[derive(Debug)]
pub struct Drain(Receiver<String>);

impl Drain {
    pub fn reading(pipe: Option<impl Read + Send + 'static>) -> Self {
        let (sender, drained) = channel();
        std::thread::spawn(move || {
            let mut text = String::new();
            pipe.map(|mut pipe| pipe.read_to_string(&mut text).ok());
            sender.send(text).ok();
        });
        Self(drained)
    }

    pub fn within(&self, deadline: Instant, group: ProcessGroup) -> String {
        self.0.recv_timeout(deadline.saturating_duration_since(Instant::now())).unwrap_or_else(|_| {
            group.kill();
            self.0.recv().unwrap_or_default()
        })
    }
}
