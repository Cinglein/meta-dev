mod task;

use std::env;
use std::process::ExitCode;

use crate::task::Task;

fn main() -> ExitCode {
    match env::args().nth(1).as_deref().map_or(Ok(Task::Usage), str::parse).and_then(Task::run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("{failure}");
            ExitCode::FAILURE
        }
    }
}
