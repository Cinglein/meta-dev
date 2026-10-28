use std::fmt;
use std::fs::{File, TryLockError};
use std::io::{self, Write};
use std::process::Stdio;

#[derive(Debug)]
pub struct Handle(File);

impl Handle {
    pub(crate) fn new(file: File) -> Handle {
        Handle(file)
    }

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        self.0.try_lock()
    }

    pub fn unlock(&self) -> io::Result<()> {
        self.0.unlock()
    }

    pub fn emptied(&mut self) -> io::Result<()> {
        self.0.set_len(0)
    }

    pub fn written(&mut self, text: impl fmt::Display) -> io::Result<()> {
        self.0.write_all(fmt::format(format_args!("{text}")).as_bytes())
    }

    pub fn cloned(&self) -> io::Result<Handle> {
        self.0.try_clone().map(Handle)
    }
}

impl From<Handle> for Stdio {
    fn from(handle: Handle) -> Stdio {
        Stdio::from(handle.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Door, Opening};

    const HELD: &str = "held";
    const STALE: &str = "stale";

    #[test]
    fn an_emptied_handle_leaves_its_file_empty() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(HELD);
        let mut held = Door::opened(&path, Opening::Truncate).unwrap();
        held.written(STALE).unwrap();
        let written = Door::read(&path).unwrap();
        held.emptied().unwrap();
        assert!(written == STALE && Door::read(&path).unwrap().is_empty());
    }
}
