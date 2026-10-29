use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::{Path, PathBuf};

use meta_run::{CommandLine, Environment};

use super::cause::Cause;
use super::change::Change;
use super::context::Context;
use super::decision::Replayed;
use super::deletable::Deletable;
use super::entry_kind::EntryKind;
use super::git_call::GitCall;
use super::halt::Halt;
use super::host::Host;
use super::interpreter::Interpreter;
use super::kept::Kept;
use super::look::Look;
use super::made::Made;
use super::occupancy::Occupancy;
use super::recording::Recording;
use super::replay::Replay;
use super::scene::Scene;
use super::shared_node::{LinkCount, NodeId, SharedNode};
use super::thaw::{Thaw, Thawed};
use super::turn::Turn;
use super::vetted::{Checkouts, RefList, Vetted};
use crate::request::local::local_clone::quarantine::Stamp;
use crate::request::local::recorded_worktree::RecordedWorktree;

struct Case(&'static str, fn(&mut Recording<'_>));

const PATH: &str = Scene::PATH;
const ESCAPES: &str = "path escapes the worktree root";
const UNLINKED: &str = "not a linked worktree of this clone";
const BORROWED: &str = "borrowed objects";
const NESTED: &str = "nested repo";
const NOT_PLAIN: &str = "index entry not plain";
const SUBMODULES: &str = "has submodules";
const IGNORED: &str = "ignored files besides cargo target dirs";
const CARGO_TAG: &str = Scene::CARGO_TAG;
const FOREIGN_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n# made by another tool\n";
const UNSIGNED_TAG: &str = "Signature: none\n# created by cargo\n";
const PAUSED: &str = "am, rebase or cherry-pick in progress";
const REDIRECTED: &str = "worktree config sets a remote";
const OFF_BRANCH: &str = "HEAD is not on the recorded branch";
const NO_BRANCH: &str = "task branch missing";
const SHARED: &str = "branch checked out elsewhere";
const STRANGER: &str = "a different worktree at the recorded path";
const UNPUSHED: &str = "commits not on the remote";
const REFS: &str = "refs/worktree/";
const OWN_CHECKOUTS: &str = "worktree /clone\nHEAD c0\nbranch refs/heads/main\n\nworktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n";
const REFUSED: [Case; 56] = [
    Case("symlink", |host| host.entry_is("/trees/task-00000000000000b1", EntryKind::Symlink)),
    Case("not found", |host| host.entry_is("/trees/task-00000000000000b1", EntryKind::Missing)),
    Case("unreadable entry", |host| host.entry_is("/trees/task-00000000000000b1", EntryKind::File)),
    Case(ESCAPES, |host| host.moved("/trees/task-00000000000000b1", "/elsewhere/x")),
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
    Case(BORROWED, |host| host.lent("/trees/peer/.git/objects", "# lent\n../../../task-00000000000000b1/x/objects")),
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
    Case(UNPUSHED, |host| host.answer_is("rev-list", "c9\n")),
    Case("remote unreachable: offline", |host| host.fails("ls-remote")),
    Case("files hard-linked outside the worktree", |host| {
        host.links(&["notes-a", "notes-b"], SharedNode::new(NodeId::new(1, 8), LinkCount::new(2)));
        host.links(&["notes-c"], SharedNode::new(NodeId::new(2, 8), LinkCount::new(2)));
    }),
    Case("per-worktree refs cannot be listed (reftable)", |host| {
        host.entry_is("/clone/.git/worktrees/task-00000000000000b1/reftable", EntryKind::Dir);
    }),
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
        host.lent("/clone/.git/worktrees/peer/objects", "../../../../../trees/task-00000000000000b1/x");
    }),
    Case(OFF_BRANCH, |host| host.answer_is("symbolic-ref", "refs/heads/other\n")),
    Case(NO_BRANCH, |host| host.answer_is(REFS, "")),
    Case(STRANGER, |host| {
        host.text_is("/trees/task-00000000000000b1/.git", "gitdir: /clone/.git/worktrees/other\n");
        host.text_is("/clone/.git/worktrees/other/gitdir", "/trees/task-00000000000000b1/.git\n");
        host.text_is("/clone/.git/worktrees/other/commondir", "../..\n");
    }),
    Case(SHARED, |host| {
        host.answer_is("list", "worktree /clone\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n");
    }),
    Case("no recorded worktree id", |_| {}),
];
// Pseudo refs written for real into the worktree's git dir, holding a commit that is not on the remote.
const PLANTED: [(&str, &str); 3] = [("ORIG_HEAD", UNPUSHED), ("WIP", UNPUSHED), ("MY-WIP", UNPUSHED)];
// A pause is any of these files in the worktree's git dir, whatever it holds (so dangling ones too).
const PAUSES: [&str; 4] = ["CHERRY_PICK_HEAD", "MERGE_HEAD", "REVERT_HEAD", "REBASE_HEAD"];
const GIT_DIR: &str = "/clone/.git/worktrees/task-00000000000000b1/";
const ACCEPTED: [fn(&mut Recording<'_>); 6] = [
    |host| host.links(&["notes-a", "notes-b"], SharedNode::new(NodeId::new(1, 8), LinkCount::new(2))),
    |_| {},
    |host| host.answer_is("list", OWN_CHECKOUTS),
    |host| host.answer_is("ls-remote", "c1\trefs/heads/main\n"),
    |host| host.lent("/trees/peer/.git/objects", "#/../../../../task-00000000000000b1\n/x/objects"),
    |host| {
        host.answer_is("ls-files", "H 100755 a1 0\trun\0H 120000 a2 0\tdocs/link\0");
        host.walks(&["docs/link"], EntryKind::Symlink);
    },
];
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
    "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n\nworktree /clone\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n",
];
const SWEPT: [&str; 5] = ["", OFF_BRANCH, OFF_BRANCH, OFF_BRANCH, SHARED];
const SURVEY_WORDS: [&str; 3] = ["list", "symbolic-ref", "ls-remote"];
const UNLISTED: &str = "cannot inspect: offline";
const STAMP: &str = "1x0";
const QUARANTINE: &str = "/trees/.gc-task-00000000000000b1-1x0";
const LATE: &str = "late.log";
const INHERITED: [(&str, &str); 3] = [("GIT_DIR", "d"), ("HOME", "h"), ("GIT_INDEX_FILE", "i")];
const ISOLATED: [(&str, Option<&str>); 6] = [
    ("GIT_CONFIG_GLOBAL", Some("/dev/null")),
    ("GIT_CONFIG_NOSYSTEM", Some("1")),
    ("GIT_CONFIG_SYSTEM", Some("/dev/null")),
    ("GIT_DIR", None),
    ("GIT_INDEX_FILE", None),
    ("GIT_TERMINAL_PROMPT", Some("0")),
];
const STAT_PINS: [&str; 2] = ["core.trustctime=true", "core.checkStat=default"];
const LEDGER: &str = "/clone/.git/meta-local/quarantine/.gc-task-00000000000000b1-1x0.json";
const WRITABLE: u32 = 0o644;
const RESTORE_STEPS: [&str; 2] = ["worktree unlock", "worktree move"];

// One real repo per test binary would need a static; each test builds its own Scene instead.
struct Bench {
    scene: Scene,
}

impl Bench {
    fn new() -> Self {
        Self { scene: Scene::new() }
    }

    fn recording(&self) -> Recording<'_> {
        let worktree = self.scene.worktree();
        Recording::of(&self.scene, self.scene.context(), |context| {
            let (worktree, context) = (worktree.clone(), context.clone());
            Replayed::new(move |replay| Vetted::proved(&worktree, &context, replay))
        })
    }

    fn vetted(recording: &Recording<'_>, worktree: &RecordedWorktree) -> Result<Vetted, Kept> {
        let context = recording.context();
        let (answer, _) = recording.answered(&Replayed::new(|replay| Vetted::proved(worktree, context, replay)));
        answer.unwrap_or_else(|| Err(Kept::Uninspected(Cause::new("stalled"))))
    }

    fn shown(vetted: Result<Vetted, Kept>) -> String {
        vetted.err().map(|kept| kept.to_string()).unwrap_or_default()
    }

    fn planted(&self, name: &str, content: &str) -> String {
        let file = self.scene.real("/clone/.git/worktrees/task-00000000000000b1").join(name);
        std::fs::write(&file, content).unwrap();
        let host = Host::default();
        let (worktree, context) = (self.scene.worktree(), self.scene.context());
        let vetted = Interpreter::new(&host).ran(&Replayed::new(|replay| Vetted::proved(&worktree, &context, replay)));
        std::fs::remove_file(&file).unwrap();
        Self::shown(vetted.and_then(|vetted| vetted))
    }

    fn wip(&self) -> String {
        self.scene.git(&self.scene.real("/clone"), &["rev-parse", "refs/hidden/wip"])
    }

    fn bytes(&self) -> String {
        let path = self.scene.real(PATH);
        let sizes = [".git", "meta.toml", "target/CACHEDIR.TAG"].map(|name| std::fs::metadata(path.join(name)).unwrap().len());
        format!("{} bytes", sizes.iter().sum::<u64>())
    }

    // Remove the vetted worktree for real, turn by turn; `raced` runs right after the move into quarantine.
    fn removed(&self, raced: impl Fn(&Path)) -> (Result<(), Kept>, Vec<Turn>) {
        let host = Host::default();
        let interpreter = Interpreter::new(&host);
        let (worktree, context) = (self.scene.worktree(), self.scene.context());
        let first = interpreter.ran(&Replayed::new(|replay| Vetted::proved(&worktree, &context, replay)));
        let first = first.and_then(|first| first).unwrap();
        let Ok(stamp) = STAMP.parse::<Stamp>();
        let removal = Replayed::new(|replay| {
            let again = Halt::settled(Vetted::proved(&worktree, &context, replay))?;
            Deletable::confirmed(first.clone(), again)?.removed(&worktree, &context, &stamp, replay).map(drop)
        });
        let mut journal = Vec::new();
        let quarantine = self.scene.real(QUARANTINE);
        for _ in 0..256 {
            if let Some(done) = interpreter.turned(&removal, &mut journal) {
                return (done, journal);
            }
            if let Some(Turn::Acted(changes, _)) = journal.last() {
                if changes.iter().any(|change| Self::moves(change, &quarantine)) {
                    raced(&quarantine);
                }
            }
        }
        (Err(Kept::Uninspected(Cause::new("stalled"))), journal)
    }

    fn moves(change: &Change, to: &Path) -> bool {
        match change {
            Change::Destroy(call) => call.words().last().is_some_and(|last| Path::new(last) == to),
            Change::Git(_) | Change::Freeze(..) | Change::Thaw(_) => false,
        }
    }

    fn ran(journal: &[Turn], words: &[&str]) -> bool {
        journal.iter().any(|turn| match turn {
            Turn::Acted(changes, made) => changes.iter().zip(made).any(|(change, made)| {
                let call = match change {
                    Change::Git(call) | Change::Destroy(call) => Some(call),
                    Change::Freeze(..) | Change::Thaw(_) => None,
                };
                call.is_some_and(|call| words.iter().zip(call.words()).all(|(word, said)| said == word))
                    && made != &Made::Git(Err(Kept::Stale))
            }),
            Turn::Looked(..) => false,
        })
    }

    fn acted(journal: &[Turn], wanted: fn(&Change) -> bool) -> bool {
        journal.iter().any(|turn| matches!(turn, Turn::Acted(changes, _) if changes.iter().any(wanted)))
    }
}

#[test]
fn only_a_clean_pushed_linked_worktree_under_the_root_is_vetted_and_its_bytes_include_its_target_dir() {
    let bench = Bench::new();
    let recorded = bench.recording();
    let vetted = ACCEPTED.map(|tamper| {
        let mut recording = recorded.clone();
        tamper(&mut recording);
        Bench::vetted(&recording, &bench.scene.worktree()).map(|vetted| vetted.bytes().to_string())
    });
    assert_eq!(vetted, ACCEPTED.map(|_| Ok(bench.bytes())));
}

#[test]
fn every_hazard_keeps_the_worktree_with_its_reason() {
    let bench = Bench::new();
    let recorded = bench.recording();
    let shown: Vec<String> = REFUSED
        .iter()
        .map(|Case(reason, tamper)| {
            let mut recording = recorded.clone();
            tamper(&mut recording);
            let worktree = if *reason == "no recorded worktree id" {
                RecordedWorktree::new(bench.scene.real(PATH), Scene::TASK.parse().unwrap())
            } else {
                bench.scene.worktree()
            };
            Bench::shown(Bench::vetted(&recording, &worktree))
        })
        .collect();
    assert_eq!(shown, REFUSED.map(|Case(reason, _)| reason));
}

#[test]
fn a_held_pseudo_ref_in_the_real_git_dir_or_any_pause_file_keeps_the_worktree() {
    let bench = Bench::new();
    let wip = bench.wip();
    let shown = PLANTED.iter().map(|(name, _)| bench.planted(name, &wip));
    assert_eq!(shown.collect::<Vec<_>>(), PLANTED.map(|(_, reason)| reason));
    let recorded = bench.recording();
    let paused = PAUSES.map(|pause| {
        let mut recording = recorded.clone();
        recording.entry_is(&[GIT_DIR, pause].concat(), EntryKind::File);
        Bench::shown(Bench::vetted(&recording, &bench.scene.worktree()))
    });
    assert_eq!(paused, PAUSES.map(|_| PAUSED));
}

#[test]
fn only_a_worktree_that_vets_the_same_again_becomes_deletable() {
    let bench = Bench::new();
    let clean = bench.recording();
    let mut moved = clean.clone();
    let tip = bench.scene.git(&bench.scene.real("/clone"), &["rev-parse", Scene::TASK]);
    moved.answer_is(REFS, &MOVED_REFS.replace("c1", tip.trim()));
    let worktree = bench.scene.worktree();
    let agains = [Bench::vetted(&clean, &worktree), Bench::vetted(&moved, &worktree), Err(Kept::Locked)];
    let shown = agains.into_iter().map(|again| {
        let first = Bench::vetted(&clean, &worktree).unwrap();
        Deletable::confirmed(first, again).err().map(|kept| kept.to_string()).unwrap_or_default()
    });
    assert_eq!(shown.collect::<Vec<_>>(), CONFIRMATIONS);
}

#[test]
fn only_open_files_under_the_worktree_block_it_and_a_failed_scan_blocks_every_worktree() {
    let scans = OPEN.map(|open| Occupancy::Listed(open.iter().map(PathBuf::from).collect()));
    let scans = [&scans[..], &[Occupancy::Unlisted(Cause::new(NO_LSOF))]].concat();
    let shown =
        scans.iter().map(|scan| scan.vacant(Path::new(PATH)).err().map(|kept| kept.to_string()).unwrap_or_default());
    assert_eq!(shown.collect::<Vec<_>>(), VACANCIES);
}

#[test]
fn a_sweep_judges_head_and_sharing_from_its_one_listing_and_reads_no_listing_head_or_remote_itself() {
    let bench = Bench::new();
    let recording = bench.recording();
    let worktree = bench.scene.worktree();
    let context = recording.context();
    let remote: Result<RefList, Kept> = Ok("".parse().unwrap());
    let swept = |checkouts: Result<Checkouts, Kept>| {
        let decision = Replayed::new(|replay| Vetted::proved_among(&worktree, context, replay, &checkouts, &remote));
        let (answer, journal) = recording.answered(&decision);
        let asked = Recording::looks(&journal).iter().any(|look| match look {
            Look::Git(call) => call.words().iter().any(|word| SURVEY_WORDS.iter().any(|survey| word == survey)),
            _ => false,
        });
        (Bench::shown(answer.unwrap()), asked)
    };
    let shown = LISTINGS.map(|listing| {
        let Ok(checkouts) = bench.scene.translated(listing).parse::<Checkouts>();
        swept(Ok(checkouts))
    });
    assert_eq!(shown, SWEPT.map(|reason| (reason.to_owned(), false)));
    assert_eq!(swept(Err(Kept::Uninspected(Cause::new("offline")))).0, UNLISTED);
}

#[test]
fn a_file_or_commit_made_after_the_vet_is_caught_in_quarantine_by_a_fresh_read_and_the_worktree_moves_back() {
    let races: [(&str, fn(&Bench, &Path)); 2] = [
        ("untracked files", |_, quarantine| std::fs::write(quarantine.join(LATE), "late").unwrap()),
        ("changed during checks", |bench, quarantine| {
            bench.scene.git(quarantine, &["commit", "-q", "--allow-empty", "-m", "late"]);
        }),
    ];
    for (reason, race) in races {
        let bench = Bench::new();
        let (removed, journal) = bench.removed(|quarantine| race(&bench, quarantine));
        assert_eq!(removed.err().map(|kept| kept.to_string()).as_deref(), Some(reason));
        assert!(bench.scene.real(PATH).join("meta.toml").exists() && !bench.scene.real(QUARANTINE).exists());
        assert!(!Bench::ran(&journal, &["worktree", "remove"]), "{journal:?}");
        assert!(Bench::acted(&journal, |change| matches!(change, Change::Freeze(..))));
        assert!(Bench::acted(&journal, |change| matches!(change, Change::Thaw(_))));
    }
}

#[test]
fn a_clean_vetted_worktree_is_removed_and_its_branch_deleted_after_a_fresh_look() {
    let bench = Bench::new();
    let (removed, journal) = bench.removed(|_| {});
    let branches = bench.scene.git(&bench.scene.real("/clone"), &["branch", "--list", Scene::TASK]);
    assert!(removed.is_ok() && !bench.scene.real(PATH).exists() && branches.is_empty(), "{journal:?}");
    let destroys: Vec<usize> = journal
        .iter()
        .enumerate()
        .filter(|(_, turn)| matches!(turn, Turn::Acted(changes, _) if changes.iter().any(Change::destroys)))
        .map(|(at, _)| at)
        .collect();
    let looked_before =
        destroys.iter().all(|at| matches!(at.checked_sub(1).and_then(|before| journal.get(before)), Some(Turn::Looked(..))));
    assert!(destroys.len() == 2 && looked_before, "{journal:?}");
}

#[test]
fn an_open_file_before_or_after_the_move_or_an_occupied_quarantine_keeps_the_worktree() {
    let busy = Bench::new();
    let _held = File::open(busy.scene.real(PATH).join("meta.toml")).unwrap();
    let (refused, journal) = busy.removed(|_| {});
    assert!(!Bench::ran(&journal, &["worktree", "move"]), "{journal:?}");
    let followed = Bench::new();
    let late = std::sync::Mutex::new(None);
    let (followed_refused, _) =
        followed.removed(|quarantine| *late.lock().unwrap() = Some(File::open(quarantine.join("meta.toml")).unwrap()));
    let occupied = Bench::new();
    std::fs::create_dir(occupied.scene.real(QUARANTINE)).unwrap();
    let (occupied_refused, _) = occupied.removed(|_| {});
    let shown = [refused, followed_refused, occupied_refused].map(|refused| refused.err().map(|kept| kept.to_string()));
    assert_eq!(shown, ["in use", "in use", "changed during checks"].map(|reason| Some(reason.to_owned())));
    assert!(followed.scene.real(PATH).exists() && !followed.scene.real(QUARANTINE).exists());
}

#[test]
fn the_interpreter_refuses_a_destroy_that_does_not_come_straight_after_a_look() {
    let bench = Bench::new();
    let common = bench.scene.real("/clone/.git");
    let path = bench.scene.real(PATH);
    let call = |words: &[&str]| {
        let words: Vec<&OsStr> = words.iter().map(OsStr::new).chain([path.as_os_str()]).collect();
        GitCall::common(&common, &words)
    };
    let host = Host::default();
    let stale = Replayed::new(|replay: &Replay<'_>| {
        replay.changed(call(&["worktree", "lock"]))??;
        replay.changed(call(&["worktree", "unlock"]))??;
        Ok(replay.destroyed(call(&["worktree", "remove"]))?)
    });
    let refused = Interpreter::new(&host).ran(&stale).and_then(|refused| refused);
    assert_eq!(refused, Ok(Err(Kept::Stale)));
    assert!(path.exists());
    let fresh = Replayed::new(|replay: &Replay<'_>| {
        replay.entry(&path)?;
        Ok(replay.destroyed(call(&["worktree", "remove"]))?)
    });
    let removed = Interpreter::new(&host).ran(&fresh).and_then(|removed| removed);
    assert!(removed.is_ok_and(|removed| removed.is_ok()) && !path.exists());
}

#[test]
fn vetting_git_ignores_inherited_git_variables_and_global_and_system_config_and_pins_stat_checks() {
    let call = GitCall::common(Path::new(QUARANTINE), &[OsStr::new(LATE)]);
    let inherited = INHERITED.map(|(name, value)| (OsString::from(name), Some(OsString::from(value))));
    let run = call.run(Environment::from(BTreeMap::from(inherited)));
    let isolated = ISOLATED.map(|(name, value)| (OsString::from(name), value.map(OsString::from)));
    assert_eq!(Environment::from(&run), Environment::from(BTreeMap::from(isolated)));
    let line = CommandLine::from(&run).to_string();
    assert!(STAT_PINS.iter().all(|pin| line.split(' ').any(|argument| argument == *pin)));
}

#[test]
fn a_stranded_quarantine_is_restored_from_its_ledger_and_without_one_nothing_moves() {
    use std::os::unix::fs::PermissionsExt as _;
    let bench = Bench::new();
    let thaw = Thaw::planned(Path::new(PATH), Thaw::ledger_of(Path::new(QUARANTINE), Path::new("/clone/.git")))
        .holding(vec![Thawed::new(PathBuf::from(QUARANTINE).join("meta.toml"), &std::fs::Permissions::from_mode(WRITABLE))]);
    let empty = Recording::empty(&bench.scene, bench.scene.context());
    let mut recording = empty.clone();
    recording.text_is(LEDGER, &serde_json::to_string(&thaw).unwrap());
    let (quarantine, common) = (bench.scene.real(QUARANTINE), bench.scene.real("/clone/.git"));
    let restore = Replayed::new(|replay| Deletable::unstranded(&quarantine, &common, replay));
    let (restored, journal) = recording.answered(&restore);
    let calls: Vec<String> = journal
        .iter()
        .flat_map(|turn| match turn {
            Turn::Acted(changes, _) => changes.clone(),
            Turn::Looked(..) => Vec::new(),
        })
        .map(|change| match change {
            Change::Git(call) | Change::Destroy(call) => {
                call.words().iter().take(2).map(|word| word.to_string_lossy().into_owned()).collect::<Vec<_>>().join(" ")
            }
            Change::Thaw(_) => "thawed".to_owned(),
            Change::Freeze(..) => "frozen".to_owned(),
        })
        .collect();
    assert_eq!(restored, Some(Ok(bench.scene.real(PATH))));
    assert_eq!(calls, [["thawed"].as_slice(), &RESTORE_STEPS].concat());
    let (missing, journal) = empty.answered(&restore);
    assert!(missing.is_some_and(|missing| missing.is_err()) && !journal.iter().any(|turn| matches!(turn, Turn::Acted(..))));
}

#[test]
fn a_decision_reads_its_journal_in_order_and_a_mismatched_journal_fails_closed() {
    let journal = [Turn::Looked(vec![Look::Entry(PathBuf::from("/a"))], vec![super::seen::Seen::Entry(EntryKind::Dir)])];
    let replay = Replay::new(&journal);
    let other = replay.entry(Path::new("/b"));
    let replay = Replay::new(&journal);
    let read = [replay.entry(Path::new("/a")), replay.entry(Path::new("/a"))];
    assert_eq!(other, Err(Replay::diverged()));
    assert_eq!(read, [Ok(EntryKind::Dir), Err(Halt::Look(vec![Look::Entry(PathBuf::from("/a"))]))]);
    let _context: Option<Context> = None;
}
