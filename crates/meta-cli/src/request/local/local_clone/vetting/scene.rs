use std::path::{Path, PathBuf};
use std::time::Duration;

use meta_run::Budget;

use super::alternates::Alternates;
use super::asked::{Beyond, PausedRoots};
use super::context::Context;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::kept::Kept;
use super::query::Query;
use super::vetted::{Guard, LocalRefs, Placed, Pointers, Settled, Survey, Tree};
use super::world::World;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::worktree_id::WorktreeId;

pub const TASK: &str = "task-00000000000000b1";
pub const PATH: &str = "/trees/task-00000000000000b1";
pub const PLACES: [&str; 3] = ["/clone", "/clone/.git", "/trees"];
pub const CARGO_TAG: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";
pub const LISTING: &str = "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n";
pub const REMOTE: &str = "c0\tHEAD\nc1\trefs/heads/task-00000000000000b1\nc0\trefs/heads/main\n";
pub const REFS: &str = "c1 refs/heads/task-00000000000000b1\n";
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
const PATIENT: Duration = Duration::from_secs(10);
const EXCLUDED: &str = "target/\n";

pub struct Recorded {
    pub survey: Survey,
    pub places: [&'static str; 3],
    pub lenders: Vec<Alternates>,
    pub walk: Vec<Found>,
    pub index: &'static str,
    pub status: &'static str,
    pub tags: Vec<(&'static str, &'static str)>,
    pub modules: EntryKind,
}

impl Recorded {
    pub fn clean() -> Self {
        let survey = Survey {
            placed: Placed::new(EntryKind::Dir, Some(PathBuf::from(PATH))),
            pointers: Pointers::new(
                EntryKind::File,
                Some(PathBuf::from("/clone/.git/worktrees/task-00000000000000b1")),
                Some(PathBuf::from("/trees/task-00000000000000b1/.git")),
                Some(PathBuf::from("/clone/.git")),
            ),
            guard: Guard::new(Vec::new(), Ok(()), "".parse().unwrap()),
            checkouts: Ok(LISTING.parse().unwrap()),
            local: Ok(Self::local(REFS, "c1")),
            remote: REMOTE.parse(),
            beyond: Self::beyond(""),
            again: None,
            settled: Settled::new(Tree::new(Vec::new()), Err(Kept::NotFound), EntryKind::Missing, Err(Kept::NotFound), Vec::new()),
        };
        let walk = WALK.iter().map(|(path, kind, bytes)| Found::new(PathBuf::from(path), *kind, ByteCount::new(*bytes)));
        Self {
            survey,
            places: PLACES,
            lenders: Vec::new(),
            walk: walk.collect(),
            index: "H 100644 a1 0\tmeta.toml\0",
            status: "!! target/\0",
            tags: vec![("target/", CARGO_TAG)],
            modules: EntryKind::Missing,
        }
    }

    pub fn local(listed: &str, head: &str) -> LocalRefs {
        LocalRefs::new(Vec::new(), vec![Some(head.to_owned())], listed.parse().unwrap())
    }

    pub fn beyond(listed: &str) -> Result<(), Kept> {
        Beyond::new(std::iter::empty(), std::iter::empty()).viewed(listed)
    }

    pub fn paused(listed: &str) -> Result<(), Kept> {
        PausedRoots.viewed(listed)
    }

    pub fn walks(&mut self, paths: &[&str], kind: EntryKind) {
        self.walk.extend(paths.iter().map(|path| Found::new(PathBuf::from(path), kind, ByteCount::default())));
    }

    pub fn settled(&self) -> Settled {
        let tags = self.tags.iter().map(|(dir, tag)| (PathBuf::from(dir), Some((*tag).to_owned()))).collect();
        let status = Ok(self.status.parse().unwrap());
        Settled::new(Tree::new(self.walk.clone()), self.index.parse(), self.modules, status, tags)
    }

    pub fn context(&self) -> Context {
        let places = self.places.map(PathBuf::from);
        let context = Context::new(places, "origin".parse().unwrap(), Budget::new(PATIENT), Ok(LISTING.to_owned().into()));
        context.seen(self.survey.remote.clone(), self.lenders.clone())
    }

    pub fn judged(&self) -> Result<super::Vetted, Kept> {
        let survey = Survey { settled: self.settled(), ..self.survey.clone() };
        survey.judged(&worktree(), &self.context())
    }
}

pub fn worktree() -> RecordedWorktree {
    RecordedWorktree::new(PathBuf::from(PATH), TASK.parse().unwrap()).identified(worktree_id())
}

pub fn worktree_id() -> WorktreeId {
    WorktreeId::parsed(Path::new(TASK)).unwrap()
}

pub struct Real {
    pub world: World,
}

impl Real {
    pub fn new(tasks: &[&str]) -> Self {
        let world = World::new(tasks, "");
        world.added(tasks);
        std::fs::create_dir_all(world.common().join("info")).unwrap();
        std::fs::write(world.common().join("info/exclude"), EXCLUDED).unwrap();
        tasks.iter().for_each(|task| {
            let target = world.trees().join(task).join("target");
            std::fs::create_dir_all(target.join("debug")).unwrap();
            std::fs::write(target.join("debug/.cargo-lock"), "").unwrap();
            std::fs::write(target.join("CACHEDIR.TAG"), CARGO_TAG).unwrap();
        });
        Self { world }
    }

    pub fn context(&self) -> Context {
        let remote = "origin".parse().unwrap();
        Context::observed(self.world.checkout(), self.world.common(), self.world.trees(), remote, Budget::new(PATIENT))
    }

    pub fn worktree(&self, task: &str) -> RecordedWorktree {
        let path = self.world.trees().join(task);
        RecordedWorktree::new(path, task.parse().unwrap()).identified(WorktreeId::parsed(Path::new(task)).unwrap())
    }

    pub fn branch(&self, task: &str) -> bool {
        World::run(self.world.clone.path(), &["rev-parse", "--verify", "-q", &format!("refs/heads/{task}")]).status.success()
    }
}
