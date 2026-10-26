use std::path::{Path, PathBuf};
use std::str::FromStr;

use task::Failure;

pub(super) struct BuiltBinary(PathBuf);

impl BuiltBinary {
    const EXECUTABLE: &str = "Executable";

    pub(super) fn path(&self) -> &Path {
        &self.0
    }
}

impl FromStr for BuiltBinary {
    type Err = Failure;

    fn from_str(line: &str) -> Result<BuiltBinary, Failure> {
        line.split_once(Self::EXECUTABLE)
            .and_then(|(_, rest)| rest.rsplit_once('('))
            .map(|(_, path)| BuiltBinary(PathBuf::from(path.trim_end_matches(')'))))
            .ok_or_else(|| Failure::Command(format!("{line} is not a cargo executable line")))
    }
}

#[cfg(test)]
mod tests {
    use super::BuiltBinary;
    const BUILT: &str = "  Executable unittests src/lib.rs (target/debug/deps/board-1a2b)";
    #[test]
    fn cargo_names_a_built_test_binary_in_parentheses_and_other_lines_are_not_binaries() {
        let built: BuiltBinary = BUILT.parse().unwrap();
        let other = "Compiling board".parse::<BuiltBinary>().is_err();
        assert!(built.path().to_str() == Some("target/debug/deps/board-1a2b") && other);
    }
}
