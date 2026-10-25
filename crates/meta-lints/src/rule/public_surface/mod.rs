mod export;
mod foreign_references;

use export::Export;
use foreign_references::ForeignReferences;

use crate::rule::binary_files::BinaryFiles;

use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

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
                        .filter(|export| references.depends_on(export.crate_name()) && !references.names(export))
                        .map(move |export| Violation::new(Site::Line(file.path().to_owned(), export.line()), &export))
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PublicSurface;
    use crate::source_file::SourceFile;

    const LIB: &str = "mod x; pub use x::{Used, Unused, Wired}; pub use x::Original as Seen;";
    const MAIN: &str = "use a::Used; fn main() { a::Seen::go(); }";
    const OWN_BINARY: &str = "use a::Wired; fn main() {}";
    const LONELY: &str = "mod y; pub use y::Lonely;";
    const FILES: [(&str, &str); 4] = [
        ("crates/a/src/lib.rs", LIB),
        ("crates/a/src/main.rs", OWN_BINARY),
        ("crates/b/src/main.rs", MAIN),
        ("crates/c/src/lib.rs", LONELY),
    ];
    const FLAGGED: &str = "Unused is exported";
    const KEPT: [&str; 4] = ["Used is exported", "Seen", "Wired", "Lonely"];

    #[test]
    fn flags_an_unnamed_reexport_but_counts_the_crates_own_binary_as_a_user() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path.to_owned(), text.to_owned()).expect("valid rust"));
        let report = PublicSurface::report(&files).to_string();
        assert!(report.contains(FLAGGED) && KEPT.iter().all(|kept| !report.contains(kept)), "{report}");
    }
}
