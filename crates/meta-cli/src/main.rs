use std::process::ExitCode;

use meta_cli::Request;
use meta_run::Run;

fn main() -> ExitCode {
    let code = match Request::try_from(std::env::args().collect::<Vec<_>>()).and_then(Request::answer) {
        Ok(output) => {
            print!("{output}");
            output.exit_code()
        }
        Err(failure) => {
            eprintln!("meta: {failure}");
            ExitCode::FAILURE
        }
    };
    Run::honor_interrupt();
    code
}
