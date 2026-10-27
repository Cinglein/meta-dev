use std::process::ExitCode;

use meta_cli::Request;

fn main() -> ExitCode {
    match Request::traced(std::env::args().collect()) {
        Ok(output) => {
            print!("{output}");
            output.exit_code()
        }
        Err(failure) => {
            eprintln!("meta: {failure}");
            ExitCode::FAILURE
        }
    }
}
