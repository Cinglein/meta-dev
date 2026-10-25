mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::ForeignReferences;

use task::BinaryFiles;

use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct PublicSurface;

impl PublicSurface {
    pub fn report(files: &[SourceFile]) -> Report {
        let binaries = BinaryFiles::collect(files);
        let references = files
            .iter()
            .map(|file| ForeignReferences::in_file(file, &binaries))
            .fold(ForeignReferences::default(), ForeignReferences::absorb);
        Report::new(
            "a crate exports exactly what another crate uses",
            files
                .iter()
                .flat_map(|file| {
                    Export::in_root(file)
                        .into_iter()
                        .filter(|export| {
                            references.depends_on(export.crate_name()) && !references.names(export)
                        })
                        .map(move |export| {
                            Violation::new(
                                Site::Line(file.path().into(), export.line()),
                                Finding::UnnamedExport {
                                    name: export.name().clone(),
                                },
                            )
                        })
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PublicSurface;
    use task::SourceFile;
    const LIB: &str =
        "mod x; use x::Private;\npub use x::{Used, Unused, Wired}; pub use x::Original as Seen;";
    const FILES: [(&str, &str, &str); 4] = [
        ("a", "crates/a/src/lib.rs", LIB),
        ("a", "crates/a/src/main.rs", "use a::Wired; fn main() {}"),
        (
            "b", "crates/b/src/main.rs", "use a::Used; fn main() { a::Seen::go(); }",
        ),
        ("c", "crates/c/src/lib.rs", "mod y; pub use y::Lonely;"),
    ];
    const REPORTED: [&str; 1] = ["crates/a/src/lib.rs:2: Unused is exported"];

    #[test]
    fn flags_an_unnamed_reexport_but_counts_the_crates_own_binary_as_a_user() {
        let files = FILES.map(|(name, path, text)| {
            SourceFile::parse(path, text.to_owned())
                .unwrap()
                .in_crate(name.parse().unwrap())
        });
        let report = PublicSurface::report(&files);
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
