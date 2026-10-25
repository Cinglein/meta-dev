use std::process::Output;

pub trait Stream {
    fn pick(output: &Output) -> String;
}
