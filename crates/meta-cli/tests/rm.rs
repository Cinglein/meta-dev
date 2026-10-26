#![allow(clippy::unwrap_used)]

mod rm_scene;

use std::fs::File;

use rm_scene::{CARGO_TAG, COMMIT, CONFIG, CacheTag, Git, ORIGIN, Refusal, Scene};

const DONE: &str = "done";
const NESTED_TARGET: &str = "done/target";
const LOOSE_TARGET: &str = "loose";
const COPY: &str = "copy";
const KINDS: [&str; 3] = ["Removing target-dir 99 bytes\n", "Removing worktree ", "Removing clone "];
const WORKTREE_PLACE: &str = " bytes project@HEAD";
const PLACES: [&str; 2] = [WORKTREE_PLACE, " bytes copy@main"];
const GIT_LINK: &str = ".git";
const BYTES_INDEX: usize = 5;
const LOGGED: [&str; 5] = ["log", "-p", "-U0", "--format=", "meta/log"];
const INTENT: &str = "\"kind\":\"Removing\"";
const WORKTREES: [&str; 3] = ["worktree", "list", "--porcelain"];
const HIDE_UNTRACKED: [&str; 3] = ["config", "status.showUntrackedFiles", "no"];
const LINK: &str = "link";
const FOREIGN: &str = "foreign";
const FOREIGN_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n# Another tool's cache.\n";
const DIRTY: &str = "dirty";
const UNTRACKED: &str = "untracked";
const IGNORED: &str = "ignored";
const UNPUSHED: &str = "unpushed";
const FORKED: &str = "forked";
const FORK: &str = "fork";
const BUSY: &str = "busy";
const HELD: &str = "held";
const HELD_INSIDE: &str = "debug/held";
const REFUSALS: [Refusal; 10] = [
    Refusal::new("missing", "not found"),
    Refusal::new(LINK, "symlink"),
    Refusal::new(".", "holds the current repo"),
    Refusal::new(FOREIGN, "neither a cargo target dir nor a git work tree root"),
    Refusal::new(DIRTY, "uncommitted changes"),
    Refusal::new(UNTRACKED, "untracked files"),
    Refusal::new(IGNORED, "ignored files besides cargo target dirs"),
    Refusal::new(UNPUSHED, "commits not on a remote"),
    Refusal::new(FORKED, "has linked worktrees"),
    Refusal::new(BUSY, "in use"),
];

#[derive(Debug, PartialEq, Eq)]
struct ShownBytes(usize);

impl std::str::FromStr for ShownBytes {
    type Err = std::num::ParseIntError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        line.split_whitespace().nth(BYTES_INDEX).unwrap_or_default().parse().map(Self)
    }
}

#[test]
fn a_target_dir_a_pushed_worktree_and_a_pushed_clone_are_removed_and_logged_without_machine_paths() {
    let scene = Scene::new();
    let worktree = Git::worktree(&scene.project(), scene.place(DONE));
    CacheTag::plant(scene.place(NESTED_TARGET), CARGO_TAG);
    let worktree_bytes = CONFIG.len() + CARGO_TAG.len() + std::fs::read(worktree.join(GIT_LINK)).unwrap().len();
    let removed = [CacheTag::plant(scene.place(LOOSE_TARGET), CARGO_TAG), worktree, scene.cloned(ORIGIN, COPY)];
    let run = scene.meta_rm(&removed);
    let worktrees = Git::run(&scene.project(), &WORKTREES).transcript();
    assert!(removed.iter().all(|path| !path.exists()) && !worktrees.contains(DONE), "{}", run.transcript());
    let logged = Git::run(&scene.place(ORIGIN), &LOGGED).transcript();
    let machine = scene.place(ORIGIN).parent().unwrap().to_str().unwrap().to_owned();
    let shown = [&KINDS[..], &PLACES[..]].concat();
    assert!(shown.iter().all(|part| run.transcript().contains(part)) && !logged.contains(&machine), "{logged}");
    let bytes = run
        .transcript()
        .lines()
        .find(|line| line.ends_with(WORKTREE_PLACE))
        .and_then(|line| line.parse::<ShownBytes>().ok());
    assert_eq!((logged.matches(INTENT).count(), bytes), (KINDS.len(), Some(ShownBytes(worktree_bytes))), "{logged}");
}

#[test]
fn each_refused_path_is_kept_with_its_reason() {
    let scene = Scene::new();
    Git::run(&scene.project(), &HIDE_UNTRACKED);
    std::os::unix::fs::symlink(scene.project(), scene.place(LINK)).unwrap();
    CacheTag::plant(scene.place(FOREIGN), FOREIGN_TAG);
    std::fs::write(Git::worktree(&scene.project(), scene.place(DIRTY)).join("meta.toml"), HELD).unwrap();
    std::fs::write(Git::worktree(&scene.project(), scene.place(UNTRACKED)).join(HELD), CONFIG).unwrap();
    std::fs::write(Git::worktree(&scene.project(), scene.place(IGNORED)).join("junk"), CONFIG).unwrap();
    Git::run(&Git::worktree(&scene.project(), scene.place(UNPUSHED)), &COMMIT);
    Git::worktree(&scene.cloned(ORIGIN, FORKED), scene.place(FORK));
    let _held = File::create(CacheTag::plant(scene.place(BUSY), CARGO_TAG).join(HELD_INSIDE)).unwrap();
    assert_eq!(scene.keeps(&REFUSALS), Ok(()));
}
