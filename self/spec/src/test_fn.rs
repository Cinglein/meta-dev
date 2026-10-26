use std::fmt;
use std::fs;
use std::path::Path;

use syn::{Item, ItemFn, ItemMod};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TestFn {
    file: &'static str,
    name: &'static str,
}

impl TestFn {
    #[must_use]
    pub const fn new(file: &'static str, name: &'static str) -> TestFn {
        TestFn { file, name }
    }

    #[must_use]
    pub fn exists_under(&self, root: &Path) -> bool {
        fs::read_to_string(root.join(self.file))
            .ok()
            .and_then(|text| syn::parse_file(&text).ok())
            .is_some_and(|parsed| self.declared_in(&parsed.items))
    }

    fn declared_in(&self, items: &[Item]) -> bool {
        items.iter().any(|item| match item {
            Item::Fn(function) => self.names_test(function),
            Item::Mod(module) => self.declared_inside(module),
            _ => false,
        })
    }

    fn names_test(&self, function: &ItemFn) -> bool {
        function.sig.ident == self.name && function.attrs.iter().any(|attribute| attribute.path().is_ident("test"))
    }

    fn declared_inside(&self, module: &ItemMod) -> bool {
        module.content.as_ref().is_some_and(|(_, inner)| self.declared_in(inner))
    }
}

impl fmt::Display for TestFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} in {}", self.name, self.file)
    }
}
