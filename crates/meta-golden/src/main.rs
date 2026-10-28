use std::process::ExitCode;

use meta_golden::Replay;

fn main() -> ExitCode {
    Replay::answered(std::env::args().collect())
}
