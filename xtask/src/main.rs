use std::env;
use std::process::ExitCode;

use xtask::Task;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let task = arguments
        .first()
        .ok_or_else(Task::usage)
        .and_then(|argument| argument.parse::<Task>().map_err(|_| Task::usage()));
    match task.and_then(|task| task.run(arguments.into_iter().skip(1))) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("{failure}");
            ExitCode::FAILURE
        }
    }
}
