use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Places {
    root: PathBuf,
    bin: PathBuf,
}

impl Places {
    const ROOT: &'static str = "$ROOT";
    const BIN: &'static str = "$BIN";

    pub fn new(root: &Path, bin: &Path) -> Self {
        let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        Self { root: canonical, bin: bin.to_path_buf() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn hidden(&self, text: &str) -> String {
        let named = [(&self.root, Self::ROOT), (&self.bin, Self::BIN)];
        let named = named.into_iter().filter(|(place, _)| !place.as_os_str().is_empty());
        named.fold(text.to_owned(), |text, (place, token)| text.replace(&*place.to_string_lossy(), token))
    }

    pub fn shown(&self, word: &str) -> OsString {
        let named = [(Self::ROOT, &self.root), (Self::BIN, &self.bin)];
        let shown = named
            .into_iter()
            .fold(word.to_owned(), |word, (token, place)| word.replace(token, &place.to_string_lossy()));
        OsString::from(shown)
    }
}
