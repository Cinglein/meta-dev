mod command;

use std::process::ExitCode;

use command::Command;

fn main() -> ExitCode {
    match Command::try_from(std::env::args().collect::<Vec<_>>()).and_then(Command::answer) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(failure) => {
            eprintln!("meta: {failure}");
            ExitCode::FAILURE
        }
    }
}
