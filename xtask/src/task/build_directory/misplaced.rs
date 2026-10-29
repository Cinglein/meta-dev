use std::io;
use std::path::PathBuf;

use thiserror::Error;
use toml_edit::{Item, TomlError};

use super::BuildDirectory;

#[derive(Debug, Error)]
pub enum Misplaced {
    #[error("{}: build.target-dir is missing; run `cargo xtask init` to set it to \"{NAME}\"", .config.display(), NAME = BuildDirectory::NAME)]
    Missing { config: PathBuf },
    #[error("{}: build.target-dir is {set}; set it to \"{NAME}\" so Spotlight skips build output", .config.display(), NAME = BuildDirectory::NAME)]
    Different { config: PathBuf, set: Box<Item> },
    #[error("{}: {error}", .config.display())]
    Unreadable { config: PathBuf, error: io::Error },
    #[error("{}: {error}", .config.display())]
    Unparsed { config: PathBuf, error: Box<TomlError> },
}
