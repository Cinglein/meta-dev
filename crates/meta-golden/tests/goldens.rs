#![allow(clippy::unwrap_used)]

use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Command, Output};

use meta_golden::{Golden, Goldens, Reach, Recording, Replay};
use meta_run::{MetaArgv, Tool as _};

const SCENARIO: &str = "injected-ident";
const OUTSIDE: &str = "outside-a-repo";
const RECORDED_LEN: usize = 6;
const TEXTS: [&str; 2] = ["a\nb", "a\nc"];
const REPLAYED: [&str; 4] =
    ["someone@example.com", "1+someone@users.noreply.github.com", "no recording answers", "recordings ran out"];
const ASKED: [&str; 2] = ["var", "GIT_AUTHOR_IDENT"];
const PROGRAMS: [&str; 2] = ["git", "gh"];
const CEILING: &str = "GIT_CEILING_DIRECTORIES";
const LINE_END: &str = "\n";
const TOPLEVEL: [&str; 2] = ["rev-parse", "--show-toplevel"];
const NO_BRANCH: [&str; 4] = ["rev-parse", "--verify", "--quiet", "refs/heads/none"];
const SCENARIO_FILE: &str = "toml";
const SCENARIOS: [&str; 4] = [
    "snapshot",
    "[[step]]\nrecord = \"list\"\nmeta = { call = \"list\" }\n",
    "online",
    "[[step]]\nrecord = \"checks\"\ngh = { call = \"commit-checks\" }\n",
];
const NO_BIN: &str = "";
const ABSENT: &str = "absent";
const STALE: [&str; 2] = ["injected-ident/99-gone.argv", "[]"];
const MATCHED: [bool; 2] = [true, false];
const UNBUILT: [&str; 2] =
    ["unbuilt.toml", "[[step]]\nrun = [\"git\", \"rev-parse\", \"--verify\", \"--quiet\", \"none\"]\n"];

#[test]
fn a_scenario_records_what_real_git_reports_as_checked_in_and_a_failed_setup_fails_the_refresh() {
    let root = tempfile::tempdir().unwrap();
    let copied = |scenario: &str| Path::new(scenario).with_extension(SCENARIO_FILE);
    let copy =
        |scenario| std::fs::copy(Goldens::workspace().dir().join(copied(scenario)), root.path().join(copied(scenario)));
    [SCENARIO, OUTSIDE].map(copy).into_iter().collect::<Result<Vec<_>, _>>().unwrap();
    let [stale, stale_argv] = STALE;
    Golden::new(&root.path().join(stale)).written(stale_argv).unwrap();
    Goldens::new(root.path()).refreshed(Path::new(NO_BIN), Reach::Offline).unwrap();
    let recorded = Goldens::new(root.path()).recordings().unwrap();
    let goldens = [Goldens::workspace(), Goldens::new(&root.path().join(NO_BIN).join(SCENARIO_FILE))];
    let matched =
        recorded.iter().map(|recording| goldens.each_ref().map(|goldens| recording.checked_in(goldens).is_ok()));
    assert_eq!(matched.collect::<Vec<_>>(), [MATCHED; RECORDED_LEN]);
    let [unbuilt, steps] = UNBUILT;
    std::fs::write(root.path().join(unbuilt), steps).unwrap();
    let rebuilt = Goldens::new(root.path()).refreshed(Path::new(NO_BIN), Reach::Offline);
    assert!(rebuilt.is_err() && recorded.iter().all(|recording| recording.viewed(&recording.out()).is_err()));
}

#[test]
fn a_golden_passes_its_own_text_and_fails_on_any_other_and_meta_runs_from_the_bin_dir() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join(SCENARIO_FILE);
    std::fs::write(&file, TEXTS[0]).unwrap();
    let golden = Golden::new(&file);
    assert!(golden.checked(TEXTS[0]).is_ok() && golden.checked(TEXTS[1]).is_err());
    let [snapshot, snapshot_steps, online, online_steps] = SCENARIOS;
    let write = |(scenario, steps): (&str, &str)| {
        std::fs::write(root.path().join(scenario).with_extension(SCENARIO_FILE), steps)
    };
    [(snapshot, snapshot_steps), (online, online_steps)].map(write).into_iter().collect::<Result<Vec<_>, _>>().unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_meta-replay"), root.path().join(MetaArgv::PROGRAM)).unwrap();
    Goldens::new(root.path()).refreshed(root.path(), Reach::Offline).unwrap();
    let recorded = Recording::all_in(&root.path().join(snapshot)).unwrap();
    assert!(
        recorded.len() == 1
            && recorded.iter().all(|recording| recording.argv().calls(MetaArgv::List))
            && !root.path().join(online).exists()
    );
}

#[test]
fn the_replayer_answers_each_call_with_the_next_recording_and_refuses_once_they_run_out() {
    let root = tempfile::tempdir().unwrap();
    let [git, gh] = PROGRAMS;
    let replayed = |scenario: &str, program: &str, asked: &[&str]| {
        let mut replay = Command::new(env!("CARGO_BIN_EXE_meta-replay"));
        let replay = replay.arg0(program).args(asked).current_dir(root.path()).env(Replay::TRAIL, root.path());
        replay.env(Replay::RECORDINGS, Goldens::workspace().dir().join(scenario)).output().unwrap()
    };
    let [author, noreply, exhausted] = [ASKED, ASKED, ASKED].map(|asked| replayed(SCENARIO, git, &asked));
    let [inside, no_branch] = [&TOPLEVEL[..], &NO_BRANCH].map(|asked| replayed(SCENARIO, git, asked));
    let [foreign, noreply_email, unanswered, ran_out] = REPLAYED;
    let shown = |answer: &Output| String::from_utf8_lossy(&answer.stdout).into_owned();
    let root_line = [root.path().canonicalize().unwrap().as_os_str().as_bytes(), LINE_END.as_bytes()].concat();
    assert!(shown(&author).contains(foreign) && shown(&noreply).contains(noreply_email) && inside.stdout == root_line);
    let real = Command::new(git).args(TOPLEVEL).current_dir(root.path()).env(CEILING, root.path()).output().unwrap();
    let [outside, other_program] =
        [(OUTSIDE, git), (SCENARIO, gh)].map(|(scenario, program)| replayed(scenario, program, &TOPLEVEL));
    assert!(
        outside.status.code() == real.status.code()
            && real.status.code() != Some(1)
            && [&author, &noreply, &inside].iter().all(|answer| answer.status.success())
            && [&exhausted, &no_branch, &other_program].iter().all(|answer| answer.status.code() == Some(1))
    );
    let mut untrailed = Command::new(env!("CARGO_BIN_EXE_meta-replay"));
    let untrailed = untrailed.arg0(git).args(ASKED).env(Replay::TRAIL, root.path().join(ABSENT));
    let untrailed = untrailed.env(Replay::RECORDINGS, Goldens::workspace().dir().join(SCENARIO)).output().unwrap();
    let complained =
        [&exhausted, &other_program, &untrailed].map(|answer| String::from_utf8_lossy(&answer.stderr).into_owned());
    let [ran_out_shown, unanswered_shown, untrailed_shown] = &complained;
    assert!(
        ran_out_shown.contains(ran_out) && unanswered_shown.contains(unanswered) && untrailed_shown.contains(ABSENT),
        "{complained:?}"
    );
}
