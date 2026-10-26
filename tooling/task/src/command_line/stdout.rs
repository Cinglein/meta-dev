use std::process::Output;

use super::stream::Stream;

pub struct Stdout;

impl Stream for Stdout {
    fn pick(output: &Output) -> String {
        String::from_utf8_lossy(&output.stdout).into_owned()
    }
}
