use std::process::ExitCode;

use meta_cli::{Call, Environment};
use meta_run::Run;

fn main() -> ExitCode {
    let directory = std::env::current_dir().unwrap_or_default();
    let program = std::env::current_exe().unwrap_or_default();
    let environment = Environment::new(directory, program, std::env::vars_os().collect());
    let call = Call::new(std::env::args().collect(), environment, Box::new(std::io::stdin()));
    Run::exit(call.answered().delivered())
}
