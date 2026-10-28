use std::path::{Path, PathBuf};

use meta_door::{Change, Door, Pair};

use super::Host;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::vetting::alternates::Alternates;
use crate::request::local::local_clone::vetting::entry_kind::EntryKind;
use crate::request::local::local_clone::vetting::found::Found;
use crate::request::local::local_clone::vetting::inspection::Inspection;

const FILE: &str = "file";
const DIR: &str = "dir";
const INNER: &str = "dir/inner";
const LINK: &str = "link";
const LINK_TARGET: &str = "file/missing";
const MISSING: &str = "missing";
const TEXT: &str = "abc";
const INNER_TEXT: &str = "ab";
const WALKED: [(&str, EntryKind, u64); 4] =
    [(DIR, EntryKind::Dir, 0), (INNER, EntryKind::File, 2), (FILE, EntryKind::File, 3), (LINK, EntryKind::Symlink, 12)];
const LENDERS: [(&str, &str); 3] = [
    ("peer/git/objects", "../../../x/objects\n"),
    ("bare/objects", "/abs/objects\n"),
    ("deep/down/repo.git/Objects", "../../../../x/objects\n"),
];
const PLAIN: &str = "lenders/plain";
const INFO: &str = "info";
const ALTERNATES: &str = "info/alternates";
const KINDS: [(&str, EntryKind); 5] = [
    (MISSING, EntryKind::Missing),
    (LINK, EntryKind::Symlink),
    (DIR, EntryKind::Dir),
    (FILE, EntryKind::File),
    (LINK_TARGET, EntryKind::Unreadable),
];
const UNLISTED: &str = "cannot inspect: ";

struct Planted(tempfile::TempDir);

impl Planted {
    fn new() -> Self {
        let planted = Self(tempfile::tempdir().unwrap());
        Door::write(planted.located(FILE), TEXT).unwrap();
        Door::changed(planted.located(DIR), Change::Dir).unwrap();
        Door::write(planted.located(INNER), INNER_TEXT).unwrap();
        Door::paired(LINK_TARGET, planted.located(LINK), Pair::Symlink).unwrap();
        Door::changed(planted.located(PLAIN), Change::DirAll).unwrap();
        LENDERS
            .iter()
            .try_for_each(|(objects, listed)| {
                Door::changed(planted.located(objects).join(INFO), Change::DirAll)?;
                Door::write(planted.located(objects).join(ALTERNATES), listed)
            })
            .unwrap();
        planted
    }

    fn located(&self, name: &str) -> PathBuf {
        Door::canonical(self.0.path()).unwrap().join(name)
    }
}

#[test]
fn a_walk_lists_every_entry_without_following_links_and_marks_what_it_cannot_read() {
    let planted = Planted::new();
    let mut walked = Host::default().walked(&planted.located(""));
    walked.retain(|found| WALKED.iter().any(|(path, ..)| found.path() == Path::new(path)));
    walked.sort_by(|left, right| left.path().cmp(right.path()));
    let expected = WALKED.map(|(path, kind, bytes)| Found::new(PathBuf::from(path), kind, ByteCount::new(bytes)));
    assert_eq!(walked, expected);
    let unreadable = [Found::new(PathBuf::new(), EntryKind::Unreadable, ByteCount::default())];
    assert_eq!(Host::default().walked(&planted.located(MISSING)), unreadable);
}

#[test]
fn entries_texts_and_canonical_paths_never_follow_a_link() {
    let planted = Planted::new();
    let kinds = KINDS.map(|(name, _)| Host::default().entry(&planted.located(name)));
    assert_eq!(kinds, KINDS.map(|(_, kind)| kind));
    let texts = [LINK, FILE].map(|name| Host::default().text(&planted.located(name)));
    assert!(texts == [None, Some(TEXT.to_owned())] && Host::default().canonical(&planted.located(MISSING)).is_none());
}

#[test]
fn neighbours_are_sibling_alternates_and_a_failed_scan_keeps_everything() {
    let planted = Planted::new();
    let neighbours = Host::default().neighbours(&planted.located(""));
    let expected = LENDERS.map(|(objects, listed)| Alternates::new(planted.located(objects), listed));
    assert!(neighbours.len() == expected.len() && expected.iter().all(|lender| neighbours.contains(lender)));
    let scanned = Host::default().occupancy(&[planted.located(MISSING)]).vacant(&planted.located(FILE));
    assert!(scanned.is_err_and(|kept| kept.to_string().starts_with(UNLISTED)));
}
