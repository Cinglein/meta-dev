use std::process::Output;

use super::stream::Stream;

#[derive(Debug)]
pub struct Stderr;

impl Stream for Stderr {
    fn pick(output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).into_owned()
    }
}
