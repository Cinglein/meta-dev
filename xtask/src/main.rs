use std::env;
use std::process::ExitCode;

use xtask::Task;

fn main() -> ExitCode {
    let task = env::args()
        .nth(1)
        .ok_or_else(Task::usage)
        .and_then(|argument| argument.parse::<Task>().map_err(|_| Task::usage()));
    match task.and_then(Task::run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("{failure}");
            ExitCode::FAILURE
        }
    }
}
