use std::io::{self, Read};
use std::process::{ExitStatus, Output};

#[derive(Debug)]
pub struct PipeBytes(String);

impl PipeBytes {
    pub fn read_all(pipe: Option<impl Read>) -> io::Result<Self> {
        let mut bytes = Vec::new();
        pipe.map(|mut pipe| pipe.read_to_end(&mut bytes)).transpose()?;
        Ok(Self(String::from_utf8_lossy(&bytes).into_owned()))
    }

    pub fn into_output(self, status: ExitStatus, stderr: Self) -> Output {
        Output { status, stdout: self.0.into_bytes(), stderr: stderr.0.into_bytes() }
    }
}
