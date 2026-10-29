use std::path::{Path, PathBuf};

use super::alternates::Alternates;
use super::cause::Cause;
use super::deletable::Deletable;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::scene::{PATH, Recorded};
use super::shared_node::{LinkCount, NodeId, SharedNode};
use super::vetted::{Guard, Placed, Pointers};
use crate::request::local::byte_count::ByteCount;

struct Case(&'static str, fn(&mut Recorded));

const ESCAPES: &str = "path escapes the worktree root";
const UNLINKED: &str = "not a linked worktree of this clone";
const BORROWED: &str = "borrowed objects";
const NESTED: &str = "nested repo";
const NOT_PLAIN: &str = "index entry not plain";
const SUBMODULES: &str = "has submodules";
const IGNORED: &str = "ignored files besides cargo target dirs";
const CARGO_TAG: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";
const FOREIGN_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n# made by another tool\n";
const UNSIGNED_TAG: &str = "Signature: none\n# created by cargo\n";
const PAUSED: &str = "am, rebase or cherry-pick in progress";
const REDIRECTED: &str = "worktree config sets a remote";
const OFF_BRANCH: &str = "HEAD is not on the recorded branch";
const NO_BRANCH: &str = "task branch missing";
const SHARED: &str = "branch checked out elsewhere";
const STRANGER: &str = "a different worktree at the recorded path";
const UNPUSHED: &str = "commits not on the remote";
const GIT_DIR: &str = "/clone/.git/worktrees/task-00000000000000b1";
const BOTH: &str = "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n\nworktree /clone\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n";
const OWN_CHECKOUTS: &str = "worktree /clone\nHEAD c0\nbranch refs/heads/main\n\nworktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n";

fn placed(scene: &mut Recorded, kind: EntryKind, canonical: &str) {
    scene.survey.placed = Placed::new(kind, Some(PathBuf::from(canonical)));
}

fn pointed(scene: &mut Recorded, entry: EntryKind, [git_dir, back, shared]: [&str; 3]) {
    scene.survey.pointers = Pointers::new(entry, Some(git_dir.into()), Some(back.into()), Some(shared.into()));
}

fn guarded(scene: &mut Recorded, present: &'static str) {
    scene.survey.guard = Guard::new(vec![present], Ok(()), "".parse().unwrap());
}

fn rooted(scene: &mut Recorded, listed: &str) {
    scene.survey.guard = Guard::new(Vec::new(), Recorded::paused(listed), "".parse().unwrap());
}

fn configured(scene: &mut Recorded, config: &str) {
    scene.survey.guard = Guard::new(Vec::new(), Ok(()), config.parse().unwrap());
}

fn unpushed(scene: &mut Recorded, listed: &str) {
    scene.survey.beyond = Recorded::beyond(listed);
}

const REFUSED: [Case; 64] = [
    Case("symlink", |scene| placed(scene, EntryKind::Symlink, PATH)),
    Case("not found", |scene| placed(scene, EntryKind::Missing, PATH)),
    Case("unreadable entry", |scene| placed(scene, EntryKind::File, PATH)),
    Case(ESCAPES, |scene| placed(scene, EntryKind::Dir, "/elsewhere/x")),
    Case(ESCAPES, |scene| scene.places = ["/clone", "/clone/.git", "/other"]),
    Case(ESCAPES, |scene| scene.places = ["/clone", "/clone/.git", PATH]),
    Case(ESCAPES, |scene| scene.places = ["/trees/task-00000000000000b1/clone", "/clone/.git", "/trees"]),
    Case(ESCAPES, |scene| scene.places = ["/clone", "/trees/task-00000000000000b1/repo", "/trees"]),
    Case(UNLINKED, |scene| pointed(scene, EntryKind::Dir, [GIT_DIR, "/trees/task-00000000000000b1/.git", "/clone/.git"])),
    Case(UNLINKED, |scene| {
        pointed(scene, EntryKind::File, ["/other/.git/worktrees/b1", "/trees/task-00000000000000b1/.git", "/clone/.git"]);
    }),
    Case(ESCAPES, |scene| pointed(scene, EntryKind::File, [GIT_DIR, "/trees/other/.git", "/clone/.git"])),
    Case(ESCAPES, |scene| pointed(scene, EntryKind::File, [GIT_DIR, "/trees/task-00000000000000b1/.git", "/clone/other"])),
    Case("locked", |scene| guarded(scene, "locked")),
    Case(BORROWED, |scene| guarded(scene, "objects/info/alternates")),
    Case(BORROWED, |scene| guarded(scene, "objects/info/alternates")),
    Case(BORROWED, |scene| {
        let lent = Alternates::new("/trees/peer/.git/objects".into(), "# lent\n../../../task-00000000000000b1/x/objects");
        scene.lenders.push(lent);
    }),
    Case(NESTED, |scene| scene.walks(&["target/debug/dep/.GIT"], EntryKind::Dir)),
    Case(NESTED, |scene| scene.walks(&["src/.Git"], EntryKind::File)),
    Case(NESTED, |scene| {
        scene.walks(&["target/x.git", "target/x.git/objects", "target/x.git/refs"], EntryKind::Dir);
        scene.walks(&["target/x.git/HEAD"], EntryKind::File);
    }),
    Case(BORROWED, |scene| scene.walks(&["target/x/Objects/Info/Alternates"], EntryKind::File)),
    Case("symlink", |scene| scene.walks(&["target/link"], EntryKind::Symlink)),
    Case("unreadable entry", |scene| scene.walks(&["target/locked"], EntryKind::Unreadable)),
    Case(NOT_PLAIN, |scene| scene.index = "S 100644 a1 0\tmeta.toml\0"),
    Case(NOT_PLAIN, |scene| scene.index = "h 100644 a1 0\tmeta.toml\0"),
    Case(NOT_PLAIN, |scene| scene.index = "H 100644 a1 1\tmeta.toml\0"),
    Case(NOT_PLAIN, |scene| scene.index = "H 040000 a1 0\tsparse/\0"),
    Case(NOT_PLAIN, |scene| scene.index = "H 100644 a1 0 meta.toml\0"),
    Case(SUBMODULES, |scene| scene.index = "H 160000 a1 0\tsub\0"),
    Case(SUBMODULES, |scene| scene.modules = EntryKind::File),
    Case(SUBMODULES, |scene| guarded(scene, "modules")),
    Case("uncommitted changes", |scene| scene.status = " M meta.toml\0!! target/\0"),
    Case("untracked files", |scene| scene.status = "?? notes\0"),
    Case(IGNORED, |scene| scene.status = "!! target/\0!! .env\0"),
    Case(IGNORED, |scene| scene.walks(&["target/notes"], EntryKind::File)),
    Case(IGNORED, |scene| scene.tags = vec![("target/", FOREIGN_TAG)]),
    Case(IGNORED, |scene| scene.tags = vec![("target/", UNSIGNED_TAG)]),
    Case(IGNORED, |scene| {
        scene.status = "!! target/\0!! out.bin\0";
        scene.walks(&["out.bin"], EntryKind::File);
        scene.tags.push(("out.bin", CARGO_TAG));
    }),
    Case("branch differs from the remote", |scene| {
        scene.survey.remote = "c2\trefs/heads/task-00000000000000b1\n".parse();
    }),
    Case(UNPUSHED, |scene| unpushed(scene, "c9\n")),
    Case("remote unreachable: offline", |scene| scene.survey.remote = Err(Kept::Unreachable(Cause::new("offline")))),
    Case(UNPUSHED, |scene| unpushed(scene, "c7\n")),
    Case(UNPUSHED, |scene| unpushed(scene, "c7\n")),
    Case(UNPUSHED, |scene| unpushed(scene, "c7\n")),
    Case("files hard-linked outside the worktree", |scene| {
        let linked = |path: &str, node| Found::new(PathBuf::from(path), EntryKind::File, ByteCount::default()).sharing(node);
        let first = SharedNode::new(NodeId::new(1, 8), LinkCount::new(2));
        let second = SharedNode::new(NodeId::new(2, 8), LinkCount::new(2));
        scene.walk.extend([linked("notes-a", first), linked("notes-b", first), linked("notes-c", second)]);
    }),
    Case("per-worktree refs cannot be listed (reftable)", |scene| guarded(scene, "reftable")),
    Case(PAUSED, |scene| rooted(scene, "CHERRY_PICK_HEAD\n")),
    Case(PAUSED, |scene| guarded(scene, "MERGE_HEAD")),
    Case(PAUSED, |scene| rooted(scene, "REVERT_HEAD\n")),
    Case(PAUSED, |scene| rooted(scene, "REBASE_HEAD\n")),
    Case(PAUSED, |scene| guarded(scene, "MERGE_HEAD")),
    Case(PAUSED, |scene| rooted(scene, "REBASE_HEAD\n")),
    Case(PAUSED, |scene| guarded(scene, "rebase-merge")),
    Case(PAUSED, |scene| guarded(scene, "rebase-apply")),
    Case(PAUSED, |scene| guarded(scene, "sequencer")),
    Case(REDIRECTED, |scene| configured(scene, "[remote \"origin\"]\n\turl = /x\n")),
    Case(REDIRECTED, |scene| configured(scene, "[URL \"/x\"]\n\tinsteadOf = /o\n")),
    Case(IGNORED, |scene| scene.walks(&["target/debug/notes.txt"], EntryKind::File)),
    Case(IGNORED, |scene| scene.walks(&["target/debug/stash", "target/debug/stash/keep"], EntryKind::Dir)),
    Case(IGNORED, |scene| scene.walks(&["target/empty"], EntryKind::Dir)),
    Case(BORROWED, |scene| {
        let objects = "/clone/.git/worktrees/peer/objects".into();
        scene.lenders.push(Alternates::new(objects, "../../../../../trees/task-00000000000000b1/x"));
    }),
    Case(OFF_BRANCH, |scene| {
        scene.survey.checkouts = Ok("worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/other\n".parse().unwrap());
    }),
    Case(NO_BRANCH, |scene| scene.survey.local = Ok(Recorded::local("", "c1"))),
    Case(STRANGER, |scene| {
        pointed(scene, EntryKind::File, ["/clone/.git/worktrees/other", "/trees/task-00000000000000b1/.git", "/clone/.git"]);
    }),
    Case(SHARED, |scene| scene.survey.checkouts = Ok(BOTH.parse().unwrap())),
];
const ACCEPTED: [fn(&mut Recorded); 6] = [
    |scene| {
        let node = SharedNode::new(NodeId::new(1, 8), LinkCount::new(2));
        let linked = |path: &str| Found::new(PathBuf::from(path), EntryKind::File, ByteCount::default()).sharing(node);
        scene.walk.extend([linked("notes-a"), linked("notes-b")]);
    },
    |_| {},
    |scene| scene.survey.checkouts = Ok(OWN_CHECKOUTS.parse().unwrap()),
    |scene| scene.survey.remote = "c1\trefs/heads/main\n".parse(),
    |scene| {
        let lent = Alternates::new("/trees/peer/.git/objects".into(), "#/../../../../task-00000000000000b1\n/x/objects");
        scene.lenders.push(lent);
    },
    |scene| {
        scene.index = "H 100755 a1 0\trun\0H 120000 a2 0\tdocs/link\0";
        scene.walks(&["docs/link"], EntryKind::Symlink);
    },
];
const VETTED: &str = "1160 bytes";
const OPEN: [&[&str]; 4] = [
    &["/trees/task-00000000000000b1/target/debug/out"],
    &["/trees/task-00000000000000b10/x"],
    &[],
    &["/clone/.git/worktrees/task-00000000000000b1/HEAD"],
];
const MOVED_REFS: &str = "c1 refs/heads/task-00000000000000b1\nc1 refs/worktree/bisect\n";
const NO_LSOF: &str = "no lsof";
const CONFIRMATIONS: [&str; 3] = ["", "changed during checks", "locked"];
const VACANCIES: [&str; 5] = ["in use", "", "", "", "cannot inspect: no lsof"];
const LISTINGS: [&str; 5] = [
    "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n",
    "",
    "worktree /trees/task-00000000000000b1\nHEAD c1\ndetached\n",
    "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/other\n",
    BOTH,
];
const HELD: [&str; 5] = ["", OFF_BRANCH, OFF_BRANCH, OFF_BRANCH, SHARED];

#[test]
fn only_a_clean_pushed_linked_worktree_under_the_root_is_vetted_and_its_bytes_include_its_target_dir() {
    let vetted = ACCEPTED.map(|tamper| {
        let mut scene = Recorded::clean();
        tamper(&mut scene);
        scene.judged().map(|vetted| vetted.bytes().to_string())
    });
    assert_eq!(vetted, ACCEPTED.map(|_| Ok(VETTED.to_owned())));
}

#[test]
fn every_hazard_keeps_the_worktree_with_its_reason() {
    let shown: Vec<String> = REFUSED
        .iter()
        .map(|Case(_, tamper)| {
            let mut scene = Recorded::clean();
            tamper(&mut scene);
            scene.judged().err().map(|kept| kept.to_string()).unwrap_or_default()
        })
        .collect();
    assert_eq!(shown, REFUSED.map(|Case(reason, _)| reason));
}

#[test]
fn only_a_worktree_that_vets_the_same_again_becomes_deletable() {
    let mut moved = Recorded::clean();
    moved.survey.local = Ok(Recorded::local(MOVED_REFS, "c1"));
    let agains = [Recorded::clean().judged(), moved.judged(), Err(Kept::Locked)];
    let shown = agains.into_iter().map(|again| {
        let first = Recorded::clean().judged().unwrap();
        Deletable::confirmed(first, again).err().map(|kept| kept.to_string()).unwrap_or_default()
    });
    assert_eq!(shown.collect::<Vec<_>>(), CONFIRMATIONS);
}

#[test]
fn a_sweep_listing_judges_head_and_sharing_and_a_failed_listing_keeps_the_worktree() {
    let held = LISTINGS.map(|listing| {
        let mut scene = Recorded::clean();
        scene.survey.checkouts = Ok(listing.parse().unwrap());
        scene.judged().err().map(|kept| kept.to_string()).unwrap_or_default()
    });
    let mut unlisted = Recorded::clean();
    unlisted.survey.checkouts = Err(Kept::Uninspected(Cause::new("offline")));
    assert!(held == HELD && unlisted.judged().is_err_and(|kept| kept.to_string() == "cannot inspect: offline"));
}

#[test]
fn only_open_files_under_the_worktree_block_it_and_a_failed_scan_blocks_every_worktree() {
    let scans = OPEN.map(|open| Occupancy::Listed(open.iter().map(PathBuf::from).collect()));
    let scans = [&scans[..], &[Occupancy::Unlisted(Cause::new(NO_LSOF))]].concat();
    let shown =
        scans.iter().map(|scan| scan.vacant(Path::new(PATH)).err().map(|kept| kept.to_string()).unwrap_or_default());
    assert_eq!(shown.collect::<Vec<_>>(), VACANCIES);
}
