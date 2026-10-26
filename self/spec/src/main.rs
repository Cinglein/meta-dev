use std::env;
use std::process::ExitCode;

use meta_spec::SpecView;
use task::SpecSettings;

fn main() -> ExitCode {
    let outcome = SpecView::file().and_then(|file| match env::args().nth(1) {
        Some(argument) if argument == SpecSettings::CHECK => SpecView::ensure_rendered(&file),
        _ => SpecView::rewrite(&file).map(drop),
    });
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("spec: {failure}");
            ExitCode::FAILURE
        }
    }
}
