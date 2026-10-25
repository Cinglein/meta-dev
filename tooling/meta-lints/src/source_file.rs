use std::fmt;
use std::fs;
use std::path::Path;

use syn::Token;
use syn::punctuated::Punctuated;

use crate::failure::Failure;
use crate::site::Site;
use crate::workspace::Workspace;

pub struct SourceFile {
    path: String,
    syntax: syn::File,
}

impl SourceFile {
    const TARGET_DIRECTORIES: [&str; 4] = ["src", "tests", "benches", "examples"];

    pub fn read(workspace: &Workspace, file: &Path) -> Result<Self, Failure> {
        let site = Site::within(workspace.root(), file);
        let path = site.file().to_owned();
        let text = fs::read_to_string(file).map_err(|error| Failure::Io { site, error })?;
        Self::parse(path, text)
    }

    pub fn parse(path: String, text: String) -> Result<Self, Failure> {
        let syntax =
            syn::parse_file(&text).map_err(|error| Failure::Parse { site: Site::File(path.clone()), error })?;
        Ok(Self { path, syntax })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn crate_name(&self) -> Option<String> {
        let parts: Vec<&str> = self.path.split('/').collect();
        parts
            .iter()
            .position(|part| Self::TARGET_DIRECTORIES.contains(part))
            .and_then(|target| target.checked_sub(1))
            .and_then(|package| parts.get(package))
            .map(|package| package.replace('-', "_"))
    }

    pub fn syntax(&self) -> &syn::File {
        &self.syntax
    }

    pub fn is_test_item(attributes: &[syn::Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && attribute.parse_args::<syn::Meta>().is_ok_and(|meta| Self::names_test(&meta))
        })
    }

    fn names_test(meta: &syn::Meta) -> bool {
        match meta {
            syn::Meta::Path(path) => path.is_ident("test"),
            syn::Meta::List(list) if list.path.is_ident("any") => list
                .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
                .is_ok_and(|metas| metas.iter().any(Self::names_test)),
            _ => false,
        }
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.path)
    }
}
