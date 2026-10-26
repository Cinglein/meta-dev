use std::process::ExitCode;

use meta_spec::SpecView;

fn main() -> ExitCode {
    match SpecView::file().and_then(|file| SpecView::rewrite(&file)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("spec: {failure}");
            ExitCode::FAILURE
        }
    }
}
