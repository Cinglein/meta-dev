mod number_sites;
mod test_files;
mod text_sites;

use number_sites::NumberSites;
use text_sites::TextSites;

use crate::rule::report::Report;
use crate::source_file::SourceFile;
use test_files::TestFiles;

pub struct PrimitiveBoundary;

impl PrimitiveBoundary {
    pub fn report(files: &[SourceFile]) -> Report {
        let tests = TestFiles::collect(files);
        Report::new(
            "primitives appear only at a type's boundary",
            files
                .iter()
                .filter(|file| !tests.contains(file.path()))
                .flat_map(|file| NumberSites::violations(file).into_iter().chain(TextSites::violations(file)))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveBoundary;
    use crate::source_file::SourceFile;

    const NEWTYPE: &str = "pub struct Depth(u8); impl Depth { pub fn plies(self) -> u8 { self.0 } }";
    const BARE: &str = "pub struct Limits { nodes: u64, index: usize, rounds: usize } impl Limits { const CAP: u32 = 1; fn read(&self) -> Option<u64> { let x: u8 = 1; self.nodes.into() } }";
    const FILES: [(&str, &str); 2] = [("crates/a/src/depth.rs", NEWTYPE), ("crates/a/src/limits.rs", BARE)];
    const FLAGGED_FILE: &str = "limits.rs";
    const FLAGGED_SITES: usize = 4;
    const KEPT: [&str; 3] = ["depth.rs", "bare u8", "usize in index"];
    const DOOR: &str = "pub struct Word; impl core::str::FromStr for Word { type Err = (); fn from_str(text: &str) -> Result<Word, ()> { text.trim().parse() } } impl Word { fn shout(&self) -> String { format!(\"{self}\") } }";
    const ROOM: &str = "pub struct Room; impl Room { fn label(&self, text: &str) -> bool { let kept = text.to_string(); text.trim().is_empty() } }";
    const TEXT_FILES: [(&str, &str); 2] = [("crates/a/src/word.rs", DOOR), ("crates/a/src/room.rs", ROOM)];
    const TEXT_FLAGGED_FILE: &str = "room.rs";
    const TEXT_FLAGGED_SITES: usize = 2;
    const TEXT_KEPT: &str = "word.rs";

    #[test]
    fn flags_declared_primitives_and_unnamed_sizes_but_not_indices_newtypes_or_locals() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path.to_owned(), text.to_owned()).expect("valid rust"));
        let report = PrimitiveBoundary::report(&files).to_string();
        assert_eq!(report.matches(FLAGGED_FILE).count(), FLAGGED_SITES, "{report}");
        assert!(KEPT.iter().all(|kept| !report.contains(kept)), "{report}");
    }

    #[test]
    fn flags_text_handling_in_a_file_without_a_parsing_or_display_impl() {
        let files =
            TEXT_FILES.map(|(path, text)| SourceFile::parse(path.to_owned(), text.to_owned()).expect("valid rust"));
        let report = PrimitiveBoundary::report(&files).to_string();
        assert_eq!(report.matches(TEXT_FLAGGED_FILE).count(), TEXT_FLAGGED_SITES, "{report}");
        assert!(!report.contains(TEXT_KEPT), "{report}");
    }
}
