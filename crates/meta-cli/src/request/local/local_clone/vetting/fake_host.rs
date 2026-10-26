use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::alternates::Alternates;
use super::cause::Cause;
use super::context::Context;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::git_call::GitCall;
use super::inspection::Inspection;
use super::kept::Kept;
use super::occupancy::Occupancy;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::worktree_id::WorktreeId;

pub const TASK: &str = "task-00000000000000b1";
pub const PATH: &str = "/trees/task-00000000000000b1";
const ROOT: &str = "/trees";
const CHECKOUT: &str = "/clone";
const COMMON: &str = "/clone/.git";
const ENTRIES: [(&str, EntryKind); 2] =
    [(PATH, EntryKind::Dir), ("/trees/task-00000000000000b1/.git", EntryKind::File)];
const TEXTS: [(&str, &str); 4] = [
    ("/trees/task-00000000000000b1/.git", "gitdir: /clone/.git/worktrees/task-00000000000000b1\n"),
    ("/clone/.git/worktrees/task-00000000000000b1/gitdir", "/trees/task-00000000000000b1/.git\n"),
    ("/clone/.git/worktrees/task-00000000000000b1/commondir", "../..\n"),
    (
        "/trees/task-00000000000000b1/target/CACHEDIR.TAG",
        "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n",
    ),
];
const ANSWERS: [(&str, &str); 10] = [
    ("symbolic-ref", "refs/heads/task-00000000000000b1\n"),
    ("ls-files", "H 100644 a1 0\tmeta.toml\0"),
    ("status", "!! target/\0"),
    ("rev-parse", "c1\n"),
    ("for-each-ref", "c1 refs/heads/task-00000000000000b1\n"),
    ("ls-remote", "c0\tHEAD\nc1\trefs/heads/task-00000000000000b1\nc0\trefs/heads/main\n"),
    ("list", ""),
    ("move", ""),
    ("remove", ""),
    ("update-ref", ""),
];
const WALK: [(&str, EntryKind, u64); 16] = [
    ("target/debug/deps", EntryKind::Dir, 0),
    ("target/debug/deps/libx.rlib", EntryKind::File, 0),
    ("target/debug/meta.d", EntryKind::File, 0),
    ("target/debug/meta.dSYM", EntryKind::Dir, 0),
    ("target/doc", EntryKind::Dir, 0),
    ("target/doc/index.html", EntryKind::File, 0),
    (".git", EntryKind::File, 50),
    ("meta.toml", EntryKind::File, 10),
    ("target", EntryKind::Dir, 0),
    ("target/CACHEDIR.TAG", EntryKind::File, 100),
    ("target/debug", EntryKind::Dir, 0),
    ("target/debug/.cargo-lock", EntryKind::File, 0),
    ("target/debug/out", EntryKind::File, 1000),
    ("target/aarch64-apple-darwin", EntryKind::Dir, 0),
    ("target/aarch64-apple-darwin/debug", EntryKind::Dir, 0),
    ("target/aarch64-apple-darwin/debug/.cargo-lock", EntryKind::File, 0),
];
const REV_LIST: &str = "rev-list";
const NOT: &str = "--not";
const IGNORE_MISSING: &str = "--ignore-missing";
const UNEXPECTED: &str = "unexpected git call";
const OFFLINE: &str = "offline";
const SPACE: &str = " ";
const MOVE: &str = "move";
const QUARANTINED: &str = ".gc-";

#[derive(Debug)]
pub struct FakeHost {
    entries: HashMap<PathBuf, EntryKind>,
    texts: HashMap<PathBuf, String>,
    moved: HashMap<PathBuf, PathBuf>,
    answers: HashMap<&'static str, Result<String, Kept>>,
    walk: Vec<Found>,
    neighbours: HashMap<PathBuf, Vec<Alternates>>,
    places: [PathBuf; 3],
    calls: RefCell<Vec<Vec<OsString>>>,
    scans: RefCell<Vec<Vec<PathBuf>>>,
    raced: HashMap<&'static str, String>,
}

impl FakeHost {
    pub fn clean() -> Self {
        Self {
            entries: ENTRIES.iter().map(|(path, kind)| (PathBuf::from(path), *kind)).collect(),
            texts: TEXTS.iter().map(|(path, text)| (PathBuf::from(path), (*text).to_owned())).collect(),
            moved: HashMap::new(),
            answers: ANSWERS.iter().map(|(word, answer)| (*word, Ok((*answer).to_owned()))).collect(),
            walk: WALK
                .iter()
                .map(|(path, kind, bytes)| Found::new(PathBuf::from(path), *kind, ByteCount::new(*bytes)))
                .collect(),
            neighbours: HashMap::new(),
            places: [ROOT, CHECKOUT, COMMON].map(PathBuf::from),
            calls: RefCell::new(Vec::new()),
            scans: RefCell::new(Vec::new()),
            raced: HashMap::new(),
        }
    }

    pub fn worktree() -> RecordedWorktree {
        RecordedWorktree::new(PathBuf::from(PATH), TASK.parse().unwrap()).identified(Self::worktree_id())
    }

    pub fn worktree_id() -> WorktreeId {
        WorktreeId::parsed(Path::new(TASK)).unwrap()
    }

    pub fn races(&mut self, race: [&'static str; 2]) {
        let [word, answer] = race;
        self.raced.insert(word, answer.to_owned());
    }

    pub fn context(&self) -> Context {
        let [root, checkout, common] = self.places.clone();
        Context::new(checkout, common, root, "origin".parse().unwrap())
    }

    pub fn entry_is(&mut self, path: &str, kind: EntryKind) {
        self.entries.insert(PathBuf::from(path), kind);
    }

    pub fn text_is(&mut self, path: &str, text: &str) {
        self.texts.insert(PathBuf::from(path), text.to_owned());
    }

    pub fn answer_is(&mut self, word: &'static str, answer: &str) {
        self.answers.insert(word, Ok(answer.to_owned()));
    }

    pub fn fails(&mut self, word: &'static str) {
        self.answers.insert(word, Err(Kept::Uninspected(Cause::new(OFFLINE))));
    }

    pub fn walks(&mut self, paths: &[&str], kind: EntryKind) {
        self.walk.extend(paths.iter().map(|path| Found::new(PathBuf::from(path), kind, ByteCount::default())));
    }

    pub fn moved(&mut self, from: &str, to: PathBuf) {
        self.moved.insert(PathBuf::from(from), to);
    }

    pub fn placed(&mut self, places: [&str; 3]) {
        self.places = places.map(PathBuf::from);
    }

    pub fn lent(&mut self, alternates: Alternates) {
        self.lent_in(ROOT, alternates);
    }

    pub fn lent_in(&mut self, root: &str, alternates: Alternates) {
        self.neighbours.entry(PathBuf::from(root)).or_default().push(alternates);
    }

    pub fn calls(&self) -> Vec<String> {
        let calls = self.calls.borrow();
        calls.iter().map(|call| call.join(OsStr::new(SPACE)).to_string_lossy().into_owned()).collect()
    }

    pub fn scans(&self) -> Vec<Vec<PathBuf>> {
        self.scans.borrow().clone()
    }

    fn quarantined(&self) -> bool {
        self.calls.borrow().iter().any(|earlier| earlier.iter().any(|argument| argument == MOVE))
    }

    fn beyond(arguments: &[OsString]) -> String {
        let listed = arguments.iter().skip_while(|argument| *argument != IGNORE_MISSING).skip(1);
        let positive: Vec<&OsString> = listed.clone().take_while(|argument| *argument != NOT).collect();
        let negative: Vec<&OsString> = listed.skip_while(|argument| *argument != NOT).collect();
        let unreached = positive.into_iter().find(|object| !negative.contains(object));
        unreached.map(|object| [object.to_string_lossy().as_ref(), "\n"].concat()).unwrap_or_default()
    }
}

impl Inspection for FakeHost {
    fn entry(&self, path: &Path) -> EntryKind {
        let texted = self.texts.contains_key(path).then_some(EntryKind::File);
        self.entries.get(path).copied().or(texted).unwrap_or(EntryKind::Missing)
    }

    fn canonical(&self, path: &Path) -> Option<PathBuf> {
        let resolved = Alternates::resolved(path);
        Some(self.moved.get(&resolved).cloned().unwrap_or(resolved))
    }

    fn text(&self, path: &Path) -> Option<String> {
        let moved_back = path.components().map(|part| {
            let name = part.as_os_str().to_string_lossy();
            PathBuf::from(name.strip_prefix(QUARANTINED).unwrap_or(&name))
        });
        self.texts.get(&moved_back.collect::<PathBuf>()).cloned()
    }

    fn git(&self, call: &GitCall) -> Result<String, Kept> {
        let arguments = call.arguments();
        let word = arguments.iter().find_map(|argument| self.answers.keys().find(|word| argument == **word));
        let raced = word.and_then(|word| self.raced.get(word)).filter(|_| self.quarantined());
        let raced = raced.map(|race| Ok(race.clone()));
        self.calls.borrow_mut().push(arguments.clone());
        let answer = raced.or_else(|| word.and_then(|word| self.answers.get(word).cloned()));
        let listed = arguments.iter().any(|argument| argument == REV_LIST).then(|| Ok(Self::beyond(&arguments)));
        answer.or(listed).unwrap_or_else(|| Err(Kept::Uninspected(Cause::new(UNEXPECTED))))
    }

    fn walked(&self, _path: &Path) -> Vec<Found> {
        self.walk.clone()
    }

    fn neighbours(&self, root: &Path) -> Vec<Alternates> {
        self.neighbours.get(root).cloned().unwrap_or_default()
    }

    fn occupancy(&self, paths: &[PathBuf]) -> Occupancy {
        self.scans.borrow_mut().push(paths.to_vec());
        Occupancy::Listed(Vec::new())
    }
}
