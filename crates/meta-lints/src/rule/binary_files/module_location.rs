use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use syn::ItemMod;

use crate::source_file::SourceFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleLocation(String);

impl ModuleLocation {
    const BINARY_ROOT: &str = "/src/main.rs";
    const LIBRARY_ROOT: &str = "/src/lib.rs";

    pub fn is_binary_root(&self) -> bool {
        self.0.ends_with(Self::BINARY_ROOT)
    }

    pub fn is_library_root(&self) -> bool {
        self.0.ends_with(Self::LIBRARY_ROOT)
    }

    pub fn declared(&self, module: &ItemMod) -> [String; 2] {
        let parent = Path::new(&self.0);
        let directory = if parent.ends_with("main.rs") || parent.ends_with("mod.rs") {
            parent.parent().map(Path::to_path_buf)
        } else {
            parent.with_extension("").into()
        }
        .unwrap_or_default();
        let name = module.ident.to_string();
        [
            directory.join(format!("{name}.rs")).display().to_string(),
            directory.join(&name).join("mod.rs").display().to_string(),
        ]
    }
}

impl FromStr for ModuleLocation {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<ModuleLocation, Infallible> {
        Ok(ModuleLocation(text.to_owned()))
    }
}

impl From<&SourceFile> for ModuleLocation {
    fn from(file: &SourceFile) -> ModuleLocation {
        let Ok(location) = file.path().parse();
        location
    }
}
