use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use task::Failure;
use task::Shard;
use task::Site;

pub struct OutputDirectory(PathBuf);

impl OutputDirectory {
    const ROOT: &str = "mutants";

    pub fn cleared(target: &Path) -> Result<OutputDirectory, Failure> {
        let root = target.join(Self::ROOT);
        match fs::remove_dir_all(&root) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(Failure::Io { site: Site::File(root), error }),
            _ => Ok(OutputDirectory(root)),
        }
    }

    pub fn root(&self) -> &Path {
        &self.0
    }

    pub fn prepared_for(&self, run: &Shard) -> Result<PathBuf, Failure> {
        let crates: Vec<String> = run.crates().iter().map(ToString::to_string).collect();
        let directory = self.0.join(crates.join("+"));
        fs::create_dir_all(&directory).map_err(|error| Failure::Io { site: Site::File(directory.clone()), error })?;
        Ok(directory)
    }
}

#[cfg(test)]
mod tests {
    use super::OutputDirectory as Output;
    use std::{env, fs, process};
    const STALE: &str = "mutants/stale/missed.txt";
    const RUN: &str = "schema+state";

    #[test]
    fn clearing_drops_stale_output_and_a_run_gets_a_directory_named_after_its_crates() {
        let root = env::temp_dir().join(format!("meta-output-{}", process::id()));
        fs::create_dir_all(root.join(STALE).parent().unwrap()).unwrap();
        fs::write(root.join(STALE), STALE).unwrap();
        let output = Output::cleared(&root).unwrap();
        let prepared = output.prepared_for(&RUN.parse().unwrap()).unwrap();
        fs::write(prepared.join(RUN), RUN).unwrap();
        assert!(!root.join(STALE).exists() && prepared.is_dir() && prepared.ends_with(RUN));
        assert!(Output::cleared(&prepared).is_ok() && Output::cleared(&prepared.join(RUN)).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
