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

    pub fn cloned(&self) -> io::Result<Handle> {
        self.0.try_clone().map(Handle)
    }
}

impl Write for Handle {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl From<Handle> for Stdio {
    fn from(handle: Handle) -> Stdio {
        Stdio::from(handle.0)
    }
}
