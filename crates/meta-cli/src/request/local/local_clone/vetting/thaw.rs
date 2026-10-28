mod thawed;

use std::path::{Path, PathBuf};

use meta_door::{Entry, MetaStore};
use serde::{Deserialize, Serialize};

pub use thawed::Thawed;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thaw {
    origin: PathBuf,
    ledger: PathBuf,
    files: Vec<Thawed>,
}

impl Thaw {
    const LEDGERS: &'static str = "meta-local/quarantine";
    const EXTENSION: &'static str = "json";

    pub fn planned(origin: &Path, ledger: PathBuf) -> Self {
        Self { origin: origin.to_path_buf(), ledger, files: Vec::new() }
    }

    pub fn ledger_of(quarantine: &Path, common: &Path) -> PathBuf {
        common.join(Self::LEDGERS).join(quarantine.file_name().unwrap_or_default()).with_extension(Self::EXTENSION)
    }

    pub fn holding(self, files: Vec<Thawed>) -> Self {
        Self { files, ..self }
    }

    pub fn origin(&self) -> &Path {
        &self.origin
    }

    pub fn ledger(&self) -> std::io::Result<Entry> {
        Self::stored(&self.ledger).unwrap_or_else(|| Ok(Entry::from(&self.ledger)))
    }

    pub fn stored(ledger: &Path) -> Option<std::io::Result<Entry>> {
        let ledgers = ledger.parent()?;
        let common = ledgers.ancestors().find(|dir| dir.join(Self::LEDGERS) == ledgers)?;
        let store = MetaStore::located(common.parent().unwrap_or(common));
        Some(store.map(|store| store.entry(ledger.strip_prefix(store.common()).unwrap_or(ledger))))
    }

    pub fn files(&self) -> &[Thawed] {
        &self.files
    }
}
