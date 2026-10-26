use std::path::PathBuf;

use super::alternates::Alternates;
use super::cause::Cause;
use super::deletable::Deletable;
use super::entry_kind::EntryKind;
use super::fake_host::FakeHost;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::vetted::Vetted;

struct Case(&'static str, fn(&mut FakeHost));

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
const OWN_CHECKOUTS: &str = "worktree /clone\nHEAD c0\nbranch refs/heads/main\n\nworktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n";
const REFUSED: [Case; 60] = [
    Case("symlink", |host| host.entry_is("/trees/task-00000000000000b1", EntryKind::Symlink)),
    Case("not found", |host| host.entry_is("/trees/task-00000000000000b1", EntryKind::Missing)),
    Case("unreadable entry", |host| host.entry_is("/trees/task-00000000000000b1", EntryKind::File)),
    Case(ESCAPES, |host| host.moved("/trees/task-00000000000000b1", PathBuf::from("/elsewhere/x"))),
    Case(ESCAPES, |host| host.placed(["/other", "/clone", "/clone/.git"])),
    Case(ESCAPES, |host| host.placed(["/trees/task-00000000000000b1", "/clone", "/clone/.git"])),
    Case(ESCAPES, |host| host.placed(["/trees", "/trees/task-00000000000000b1/clone", "/clone/.git"])),
    Case(ESCAPES, |host| host.placed(["/trees", "/clone", "/trees/task-00000000000000b1/repo"])),
    Case(UNLINKED, |host| host.entry_is("/trees/task-00000000000000b1/.git", EntryKind::Dir)),
    Case(UNLINKED, |host| host.text_is("/trees/task-00000000000000b1/.git", "gitdir: /other/.git/worktrees/b1\n")),
    Case(ESCAPES, |host| host.text_is("/clone/.git/worktrees/task-00000000000000b1/gitdir", "/trees/other/.git\n")),
    Case(ESCAPES, |host| host.text_is("/clone/.git/worktrees/task-00000000000000b1/commondir", "../../../other\n")),
    Case("locked", |host| host.entry_is("/clone/.git/worktrees/task-00000000000000b1/locked", EntryKind::File)),
    Case(BORROWED, |host| host.entry_is("/clone/.git/objects/info/alternates", EntryKind::File)),
    Case(BORROWED, |host| {
        host.entry_is("/clone/.git/worktrees/task-00000000000000b1/objects/info/alternates", EntryKind::File);
    }),
    Case(BORROWED, |host| {
        host.lent(Alternates::new(
            "/trees/peer/.git/objects".into(),
            "# lent\n../../../task-00000000000000b1/x/objects",
        ));
    }),
    Case(NESTED, |host| host.walks(&["target/debug/dep/.GIT"], EntryKind::Dir)),
    Case(NESTED, |host| host.walks(&["src/.Git"], EntryKind::File)),
    Case(NESTED, |host| {
        host.walks(&["target/x.git", "target/x.git/objects", "target/x.git/refs"], EntryKind::Dir);
        host.walks(&["target/x.git/HEAD"], EntryKind::File);
    }),
    Case(BORROWED, |host| host.walks(&["target/x/Objects/Info/Alternates"], EntryKind::File)),
    Case("symlink", |host| host.walks(&["target/link"], EntryKind::Symlink)),
    Case("unreadable entry", |host| host.walks(&["target/locked"], EntryKind::Unreadable)),
    Case(NOT_PLAIN, |host| host.answer_is("ls-files", "S 100644 a1 0\tmeta.toml\0")),
    Case(NOT_PLAIN, |host| host.answer_is("ls-files", "h 100644 a1 0\tmeta.toml\0")),
    Case(NOT_PLAIN, |host| host.answer_is("ls-files", "H 100644 a1 1\tmeta.toml\0")),
    Case(NOT_PLAIN, |host| host.answer_is("ls-files", "H 040000 a1 0\tsparse/\0")),
    Case(NOT_PLAIN, |host| host.answer_is("ls-files", "H 100644 a1 0 meta.toml\0")),
    Case(SUBMODULES, |host| host.answer_is("ls-files", "H 160000 a1 0\tsub\0")),
    Case(SUBMODULES, |host| host.entry_is("/trees/task-00000000000000b1/.gitmodules", EntryKind::File)),
    Case(SUBMODULES, |host| host.entry_is("/clone/.git/modules", EntryKind::Dir)),
    Case("uncommitted changes", |host| host.answer_is("status", " M meta.toml\0!! target/\0")),
    Case("untracked files", |host| host.answer_is("status", "?? notes\0")),
    Case(IGNORED, |host| host.answer_is("status", "!! target/\0!! .env\0")),
    Case(IGNORED, |host| host.walks(&["target/notes"], EntryKind::File)),
    Case(IGNORED, |host| host.text_is("/trees/task-00000000000000b1/target/CACHEDIR.TAG", FOREIGN_TAG)),
    Case(IGNORED, |host| host.text_is("/trees/task-00000000000000b1/target/CACHEDIR.TAG", UNSIGNED_TAG)),
    Case(IGNORED, |host| {
        host.answer_is("status", "!! target/\0!! out.bin\0");
        host.walks(&["out.bin"], EntryKind::File);
        host.text_is("/trees/task-00000000000000b1/out.bin/CACHEDIR.TAG", CARGO_TAG);
    }),
    Case("branch differs from the remote", |host| {
        host.answer_is("ls-remote", "c2\trefs/heads/task-00000000000000b1\n");
    }),
    Case("commits not on the remote", |host| host.answer_is("rev-parse", "c9\n")),
    Case("remote unreachable: offline", |host| host.fails("ls-remote")),
    Case("commits not on the remote", |host| host.holds(&["ORIG_HEAD"])),
    Case("commits not on the remote", |host| host.holds(&["WIP"])),
    Case("per-worktree refs cannot be listed (reftable)", |host| {
        host.entry_is("/clone/.git/worktrees/task-00000000000000b1/reftable", EntryKind::Dir);
    }),
    Case(PAUSED, |host| host.holds(&["CHERRY_PICK_HEAD"])),
    Case(PAUSED, |host| host.holds(&["MERGE_HEAD"])),
    Case(PAUSED, |host| host.holds(&["REVERT_HEAD"])),
    Case(PAUSED, |host| host.holds(&["REBASE_HEAD"])),
    Case(PAUSED, |host| host.entry_is("/clone/.git/worktrees/task-00000000000000b1/rebase-merge", EntryKind::Dir)),
    Case(PAUSED, |host| host.entry_is("/clone/.git/worktrees/task-00000000000000b1/rebase-apply", EntryKind::Dir)),
    Case(PAUSED, |host| host.entry_is("/clone/.git/worktrees/task-00000000000000b1/sequencer", EntryKind::Dir)),
    Case(REDIRECTED, |host| {
        host.text_is(
            "/clone/.git/worktrees/task-00000000000000b1/config.worktree",
            "[remote \"origin\"]\n\turl = /x\n",
        );
    }),
    Case(REDIRECTED, |host| {
        host.text_is("/clone/.git/worktrees/task-00000000000000b1/config.worktree", "[URL \"/x\"]\n\tinsteadOf = /o\n");
    }),
    Case(IGNORED, |host| host.walks(&["target/debug/notes.txt"], EntryKind::File)),
    Case(IGNORED, |host| host.walks(&["target/debug/stash", "target/debug/stash/keep"], EntryKind::Dir)),
    Case(IGNORED, |host| host.walks(&["target/empty"], EntryKind::Dir)),
    Case(BORROWED, |host| {
        let objects = "/clone/.git/worktrees/peer/objects".into();
        host.lent_in("/clone/.git/worktrees", Alternates::new(objects, "../../../../../trees/task-00000000000000b1/x"));
    }),
    Case(OFF_BRANCH, |host| host.answer_is("symbolic-ref", "refs/heads/other\n")),
    Case(NO_BRANCH, |host| host.answer_is("for-each-ref", "")),
    Case(STRANGER, |host| {
        host.text_is("/trees/task-00000000000000b1/.git", "gitdir: /clone/.git/worktrees/other\n");
        host.text_is("/clone/.git/worktrees/other/gitdir", "/trees/task-00000000000000b1/.git\n");
        host.text_is("/clone/.git/worktrees/other/commondir", "../..\n");
    }),
    Case(SHARED, |host| {
        host.answer_is("list", "worktree /clone\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n");
    }),
];
const ACCEPTED: [fn(&mut FakeHost); 5] = [
    |_| {},
    |host| host.answer_is("list", OWN_CHECKOUTS),
    |host| host.answer_is("ls-remote", "c1\trefs/heads/main\n"),
    |host| {
        host.lent(Alternates::new(
            "/trees/peer/.git/objects".into(),
            "#/../../../../task-00000000000000b1\n/x/objects",
        ));
    },
    |host| {
        host.answer_is("ls-files", "H 100755 a1 0\trun\0H 120000 a2 0\tdocs/link\0");
        host.walks(&["docs/link"], EntryKind::Symlink);
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
const CONFIRMATIONS: [&str; 5] = ["in use", "", "changed during checks", "locked", "cannot inspect: no lsof"];

#[test]
fn only_a_clean_pushed_linked_worktree_under_the_root_is_vetted_and_its_bytes_include_its_target_dir() {
    let vetted = ACCEPTED.map(|tamper| {
        let mut host = FakeHost::clean();
        tamper(&mut host);
        Vetted::proved(&FakeHost::worktree(), &host.context(), &host).map(|vetted| vetted.bytes().to_string())
    });
    assert_eq!(vetted, ACCEPTED.map(|_| Ok(VETTED.to_owned())));
}

#[test]
fn every_hazard_keeps_the_worktree_with_its_reason() {
    let shown: Vec<String> = REFUSED
        .iter()
        .map(|Case(_, tamper)| {
            let mut host = FakeHost::clean();
            tamper(&mut host);
            Vetted::proved(&FakeHost::worktree(), &host.context(), &host)
                .err()
                .map(|kept| kept.to_string())
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(shown, REFUSED.map(|Case(reason, _)| reason));
}

#[test]
fn only_a_vacant_worktree_that_vets_the_same_again_becomes_deletable() {
    let host = FakeHost::clean();
    let mut moved = FakeHost::clean();
    moved.answer_is("for-each-ref", MOVED_REFS);
    let vet = |host: &FakeHost| Vetted::proved(&FakeHost::worktree(), &host.context(), host);
    let scans = OPEN.map(|open| Occupancy::Listed(open.iter().map(PathBuf::from).collect()));
    let scans = [&scans[..], &[Occupancy::Unlisted(Cause::new(NO_LSOF))]].concat();
    let agains = [vet(&host), vet(&host), vet(&moved), Err(Kept::Locked), vet(&host)];
    let shown = scans.iter().zip(agains).map(|(scan, again)| {
        Deletable::confirmed(vet(&host).unwrap(), scan, again).err().map(|kept| kept.to_string()).unwrap_or_default()
    });
    assert_eq!(shown.collect::<Vec<_>>(), CONFIRMATIONS);
}
