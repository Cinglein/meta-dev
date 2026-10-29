use std::process::ExitCode;

use meta_cli::{Request, TraceRecord};
use meta_run::Run;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let trace = TraceRecord::started(&args);
    let code = match Request::try_from(args).and_then(Request::answer) {
        Ok(output) => {
            print!("{output}");
            output.exit_code()
        }
        Err(failure) => {
            eprintln!("meta: {failure}");
            ExitCode::FAILURE
        }
    };
    trace.kept();
    Run::exit(code)
}
