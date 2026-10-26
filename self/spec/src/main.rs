use std::process::ExitCode;

use meta_spec::SpecView;

fn main() -> ExitCode {
    match SpecView::rewrite(&SpecView::root()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("spec: {failure}");
            ExitCode::FAILURE
        }
    }
}
