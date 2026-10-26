mod finding;
mod number_sites;
mod text_sites;

use std::path::PathBuf;

use number_sites::NumberSites;
use text_sites::TextSites;

use task::Policy;
use task::Report;
use task::SourceFile;
use task::SupportModules;

#[derive(Debug)]
pub struct PrimitiveBoundary;

impl PrimitiveBoundary {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            "primitives appear only at a type's boundary",
            files
                .iter()
                .filter(|file| !support.contains(&file.path().to_owned()))
                .flat_map(|file| {
                    NumberSites::violations(file, policy)
                        .into_iter()
                        .chain(TextSites::violations(file))
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveBoundary;
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const BARE: &str = "
pub struct Limits { nodes: u64, index: usize, rounds: usize } pub enum Kind { Depth(u8) }
impl Limits { const CAP: u32 = 1; fn read(&self) -> Option<u64> { let x: u8 = 1; self.nodes.into() } }
const LIMIT: u64 = 1; pub trait Bound { const MOST: u8; } pub type Count = u32;
mod inner { pub fn depth() -> u16 { 1 } }
#[cfg(test)] mod tests { pub fn depth() -> u16 { 1 } }
";
    const NEWTYPE: &str =
        "pub struct Depth(u8); impl Depth { pub fn plies(self) -> u8 { self.0 } }";
    const WIRE: &str = "\nuse serde_json::Value; use std::any::Any as Erased;\npub struct Settings { raw: Value, erased: Box<Erased>, table: toml::Value, local: Local } pub struct Local;";
    const RAW: &str = "use serde_json::Value; pub struct Raw(Value); impl Raw { fn inner(&self) -> &Value { &self.0 } }";
    const FILES: [(&str, &str); 5] = [
        ("crates/a/src/depth.rs", NEWTYPE),
        ("tooling/a/src/limits.rs", BARE),
        ("crates/a/src/pair.rs", "\npub struct Pair(u8, u8);"),
        ("crates/a/src/raw.rs", RAW),
        ("crates/a/src/wire.rs", WIRE),
    ];
    const REPORTED: [&str; 14] = [
        "limits.rs:2: bare u64 in nodes",
        "limits.rs:2: bare u8 in Kind",
        "limits.rs:3: bare u32 in CAP",
        "limits.rs:3: bare u64 in read",
        "limits.rs:4: bare u64 in LIMIT",
        "limits.rs:4: bare u8 in MOST",
        "limits.rs:4: bare u32 in Count",
        "limits.rs:5: bare u16 in depth",
        "limits.rs:2: usize in rounds",
        "pair.rs:2: bare u8 in Pair",
        "pair.rs:2: bare u8 in Pair",
        "wire.rs:3: serde_json::Value in raw is a wire form",
        "wire.rs:3: std::any::Any in erased",
        "wire.rs:3: toml::Value in table",
    ];
    const DOOR: &str = "pub struct Word; impl core::str::FromStr for Word { type Err = (); fn from_str(text: &str) -> Result<Word, ()> { text.trim().parse() } } impl Word { fn shout(&self) -> String { format!(\"{self}\") } }";
    const ROOM: &str = "
pub struct Room;
impl Room { fn label(&self, text: &str) -> bool { let kept = text.to_string(); text.trim().is_empty() } }
impl Room { fn greet(&self) -> String { let fresh = String::new(); let parts = Vec::from(fresh); format!(\"{fresh}\") } fn copy(&self) -> String { String::from(\"room\") } }
mod inner { fn shout(text: &str) -> String { text.trim().to_owned() } }
#[cfg(test)] mod tests { fn shout(text: &str) -> String { text.trim().to_owned() } }
#[cfg(test)] mod support;
";
    const SUPPORT: &str = "fn shout(text: &str) -> String { text.trim().to_owned() }";
    const TEXT_FILES: [(&str, &str); 3] = [
        ("crates/a/src/word.rs", DOOR),
        ("crates/a/src/room.rs", ROOM),
        ("crates/a/src/room/support.rs", SUPPORT),
    ];
    const TEXT_REPORTED: [&str; 5] = [
        "room.rs:3: to_string handles", "room.rs:3: trim handles", "room.rs:4: format handles",
        "room.rs:4: from handles", "room.rs:5: trim handles",
    ];

    #[test]
    fn flags_declared_primitives_and_text_handling_but_not_newtypes_doors_indices_locals_or_tests()
    {
        let files = FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let texts =
            TEXT_FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let report = PrimitiveBoundary::report(&files, &Policy::defaults());
        let text_report = PrimitiveBoundary::report(&texts, &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
        assert!(text_report.lists_exactly(&TEXT_REPORTED), "{text_report}");
    }
}
