use std::process::ExitCode;

use meta_cli::Request;

fn main() -> ExitCode {
    match Request::try_from(std::env::args().collect::<Vec<_>>()).and_then(Request::answer) {
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
