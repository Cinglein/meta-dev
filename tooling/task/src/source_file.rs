use meta_door::Door;
use std::path::{Path, PathBuf};

use syn::ItemMod;

use crate::crate_name::CrateName;
use crate::failure::Failure;
use crate::site::Site;
use crate::walk::Walk;
use crate::workspace::Workspace;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    crate_name: Option<CrateName>,
    text: String,
    syntax: syn::File,
}

impl SourceFile {
    pub fn read(
        workspace: &Workspace,
        file: &Path,
        crate_name: CrateName,
    ) -> Result<Self, Failure> {
        let path = workspace.relative(file);
        let text = Door::read(file).map_err(|error| Failure::Io {
            site: Site::File(path.clone()),
            error,
        })?;
        Self::parse(path, text).map(|parsed| parsed.in_crate(crate_name))
    }

    pub fn parse(path: impl Into<PathBuf>, text: String) -> Result<Self, Failure> {
        let path = path.into();
        let syntax = syn::parse_file(&text).map_err(|error| Failure::Parse {
            site: Site::File(path.clone()),
            error,
        })?;
        Ok(Self {
            path,
            crate_name: None,
            text,
            syntax,
        })
    }

    #[must_use]
    pub fn in_crate(self, crate_name: CrateName) -> SourceFile {
        SourceFile {
            crate_name: Some(crate_name),
            ..self
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn crate_name(&self) -> Option<&CrateName> {
        self.crate_name.as_ref()
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn syntax(&self) -> &syn::File {
        &self.syntax
    }

    #[must_use]
    pub fn walk(module: &ItemMod) -> Walk {
        if module.attrs.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && attribute
                    .parse_args::<syn::Ident>()
                    .is_ok_and(|ident| ident == "test")
        }) {
            Walk::Skipped
        } else {
            Walk::Entered
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CrateName, SourceFile, Walk};
    const PATH: &str = "crates/board/src/square.rs";
    const BOARD: &str = "board";
    const MODULES: &str = "#[cfg(test)] mod tests {} #[cfg(feature = \"proptest\")] mod strategies {} #[allow(test)] mod odd {}";
    const TEST_MODULES: usize = 1;

    #[test]
    fn a_file_knows_its_crate_only_once_placed_and_only_a_cfg_test_module_is_a_test_module() {
        let parsed = SourceFile::parse(PATH, String::new()).expect("empty is valid");
        let board: CrateName = BOARD.parse().unwrap();
        assert!(parsed.crate_name().is_none());
        assert_eq!(parsed.in_crate(board.clone()).crate_name(), Some(&board));
        let parsed = syn::parse_file(MODULES).expect("valid rust");
        let test_modules = parsed
            .items
            .iter()
            .filter(
                |item| matches!(item, syn::Item::Mod(module) if SourceFile::walk(module) == Walk::Skipped),
            )
            .count();
        assert_eq!(test_modules, TEST_MODULES);
    }
}
