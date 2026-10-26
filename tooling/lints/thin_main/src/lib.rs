mod finding;

use finding::Finding;
use syn::spanned::Spanned;
use syn::{Item, ItemFn};

use task::{LineNumber, Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct ThinMain;

impl ThinMain {
    const BINARY_ROOT: &str = "src/main.rs";

    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "a binary is a thin main over its library",
            files
                .iter()
                .filter(|file| file.path().ends_with(Self::BINARY_ROOT))
                .flat_map(|file| {
                    file.syntax()
                        .items
                        .iter()
                        .filter(|item| !Self::is_thin(item))
                        .map(|item| {
                            Violation::new(
                                Site::Line(file.path().into(), LineNumber::from(item.span())),
                                Finding::Logic,
                            )
                        })
                })
                .collect(),
        )
    }

    fn is_thin(item: &Item) -> bool {
        match item {
            Item::Use(_) => true,
            Item::Fn(ItemFn { sig, .. }) => sig.ident == "main",
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThinMain;
    use task::SourceFile;
    const MAIN: &str =
        "mod request; use std::process::ExitCode; struct Args; fn main() {} fn helper() {}";
    const REPORTED: [&str; 3] = ["src/main.rs:1", "src/main.rs:1", "src/main.rs:1"];

    #[test]
    fn only_imports_and_fn_main_sit_in_a_binary_root() {
        let main = SourceFile::parse("crates/a/src/main.rs", MAIN.to_owned()).unwrap();
        let library = SourceFile::parse("crates/a/src/lib.rs", MAIN.to_owned()).unwrap();
        let report = ThinMain::report(&[main, library]);
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
