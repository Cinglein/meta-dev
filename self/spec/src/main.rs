use std::env;
use std::process::ExitCode;

use meta_spec::SpecView;

fn main() -> ExitCode {
    match SpecView::file().and_then(|file| SpecView::follow(&file, env::args().nth(1).as_deref())) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("spec: {failure}");
            ExitCode::FAILURE
        }
    }
}
