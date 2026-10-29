mod misplaced;

use std::fmt;
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, value};

pub use misplaced::Misplaced;

#[derive(Debug)]
pub struct BuildDirectory {
    config: PathBuf,
    document: DocumentMut,
}

impl BuildDirectory {
    pub const NAME: &str = "target.noindex";
    const CONFIG: &str = ".cargo/config.toml";
    const TABLE: &str = "build";
    const KEY: &str = "target-dir";

    pub fn read(root: &Path) -> Result<BuildDirectory, Misplaced> {
        let config = root.join(Self::CONFIG);
        let text = match fs::read_to_string(&config) {
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            read => read.map_err(|error| Misplaced::Unreadable { config: config.clone(), error })?,
        };
        let document =
            text.parse().map_err(|error| Misplaced::Unparsed { config: config.clone(), error: Box::new(error) })?;
        Ok(BuildDirectory { config, document })
    }

    pub fn checked(&self) -> Result<(), Misplaced> {
        let config = self.config.clone();
        match self.configured_target() {
            None => Err(Misplaced::Missing { config }),
            Some(set) if set.as_str() == Some(Self::NAME) => Ok(()),
            Some(set) => Err(Misplaced::Different { config, set: Box::new(set.clone()) }),
        }
    }

    pub fn initialized(self) -> Result<(), Misplaced> {
        match self.configured_target() {
            Some(_) => self.checked(),
            None => self.merged(),
        }
    }

    fn merged(mut self) -> Result<(), Misplaced> {
        let config = self.config.clone();
        let build = self.document.entry(Self::TABLE).or_insert(Item::Table(Table::new()));
        let set = Box::new(build.clone());
        let table = build.as_table_like_mut().ok_or_else(|| Misplaced::Different { config: config.clone(), set })?;
        table.insert(Self::KEY, value(Self::NAME));
        fs::create_dir_all(config.parent().unwrap_or(&config))
            .and_then(|()| File::create(&config))
            .and_then(|mut file| write!(file, "{self}"))
            .map_err(|error| Misplaced::Unreadable { config: config.clone(), error })?;
        println!("init: {} sets build.target-dir = \"{}\"", config.display(), Self::NAME);
        Ok(())
    }

    fn configured_target(&self) -> Option<&Item> {
        self.document.get(Self::TABLE).and_then(|build| build.get(Self::KEY))
    }
}

impl fmt::Display for BuildDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.document)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::BuildDirectory;

    const CONFIG: &str = ".cargo/config.toml";
    const FIX: &str = "cargo xtask init";
    const KEPT: &str = "[alias]\nxtask = \"run --package xtask --\"\n";
    const MERGED: &str = "[alias]\nxtask = \"run --package xtask --\"\n\n[build]\ntarget-dir = \"target.noindex\"\n";
    const FOREIGN: &str = "[build]\ntarget-dir = \"out\"\n";

    #[test]
    fn init_merges_the_noindex_target_dir_once_keeps_other_keys_and_refuses_a_foreign_one() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join(CONFIG);
        assert!(BuildDirectory::read(root.path()).unwrap().checked().unwrap_err().to_string().contains(FIX));
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        fs::write(&config, KEPT).unwrap();
        (0..2).for_each(|_| BuildDirectory::read(root.path()).unwrap().initialized().unwrap());
        assert_eq!(fs::read_to_string(&config).unwrap(), MERGED);
        fs::write(&config, FOREIGN).unwrap();
        let refused = BuildDirectory::read(root.path()).unwrap().initialized().unwrap_err().to_string();
        assert!(refused.contains("\"out\"") && fs::read_to_string(&config).unwrap() == FOREIGN);
    }
}
