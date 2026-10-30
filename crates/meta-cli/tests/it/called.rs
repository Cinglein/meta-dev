use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use meta_cli::{Answer, Call, Environment};

pub struct Called(BTreeMap<OsString, OsString>);

impl Called {
    const META: &str = env!("CARGO_BIN_EXE_meta");

    pub fn new(variables: BTreeMap<OsString, OsString>) -> Self {
        Self(variables)
    }

    pub fn answered(&self, words: &[&str], directory: &Path, input: &str) -> Answer {
        let words = [Self::META].iter().chain(words).map(|word| (*word).to_owned()).collect();
        let environment = Environment::new(directory.to_path_buf(), PathBuf::from(Self::META), self.0.clone());
        Call::new(words, environment, Box::new(input.as_bytes())).answered()
    }
}
