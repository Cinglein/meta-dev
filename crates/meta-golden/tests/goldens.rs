#![allow(clippy::unwrap_used)]

use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Command, Output};

use meta_golden::{Golden, Goldens, Replay};

const SCENARIO: &str = "injected-ident";
const RECORDED_LEN: usize = 6;
const TEXTS: [&str; 2] = ["a\nb", "a\nc"];
const REPLAYED: [&str; 4] =
    ["someone@example.com", "1+someone@users.noreply.github.com", "no recording answers", "recordings ran out"];
const ASKED: [&str; 2] = ["var", "GIT_AUTHOR_IDENT"];
const PROGRAMS: [&str; 2] = ["git", "gh"];
const GIT_DIR: &str = "GIT_DIR";
const LINE_END: &str = "\n";
const TOPLEVEL: [&str; 2] = ["rev-parse", "--show-toplevel"];
const NO_BRANCH: [&str; 4] = ["rev-parse", "--verify", "--quiet", "refs/heads/none"];
const SCENARIO_FILE: &str = "toml";
const NO_BIN: &str = "";
const MATCHED: [bool; 2] = [true, false];
const UNBUILT: [&str; 2] =
    ["unbuilt.toml", "[[step]]\nrun = [\"git\", \"rev-parse\", \"--verify\", \"--quiet\", \"none\"]\n"];

#[test]
fn a_scenario_records_what_real_git_reports_as_checked_in_and_a_failed_setup_fails_the_refresh() {
    let root = tempfile::tempdir().unwrap();
    let scenario = Path::new(SCENARIO).with_extension(SCENARIO_FILE);
    std::fs::copy(Goldens::workspace().dir().join(&scenario), root.path().join(&scenario)).unwrap();
    Goldens::new(root.path()).refreshed(Path::new(NO_BIN)).unwrap();
    let recorded = Goldens::new(root.path()).recordings().unwrap();
    let checked_in = Goldens::workspace().dir().join(SCENARIO);
    let matched =
        recorded.iter().map(|recording| [&checked_in, root.path()].map(|dir| recording.checked_in(dir).is_ok()));
    assert_eq!(matched.collect::<Vec<_>>(), [MATCHED; RECORDED_LEN]);
    let [unbuilt, steps] = UNBUILT;
    std::fs::write(root.path().join(unbuilt), steps).unwrap();
    let rebuilt = Goldens::new(root.path()).refreshed(Path::new(NO_BIN));
    assert!(rebuilt.is_err() && recorded.iter().all(|recording| recording.viewed(&recording.out()).is_err()));
}

#[test]
fn a_golden_passes_its_own_text_and_fails_on_any_other() {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), TEXTS[0]).unwrap();
    let golden = Golden::new(file.path());
    assert!(golden.checked(TEXTS[0]).is_ok() && golden.checked(TEXTS[1]).is_err());
}

#[test]
fn the_replayer_answers_each_call_with_the_next_recording_and_refuses_once_they_run_out() {
    let root = tempfile::tempdir().unwrap();
    let [git, gh] = PROGRAMS;
    let replayed = |program: &str, asked: &[&str]| {
        let mut replay = Command::new(env!("CARGO_BIN_EXE_meta-replay"));
        let replay = replay.arg0(program).args(asked).current_dir(root.path()).env(Replay::TRAIL, root.path());
        replay.env(Replay::RECORDINGS, Goldens::workspace().dir().join(SCENARIO)).output().unwrap()
    };
    let [author, noreply, exhausted] = [ASKED, ASKED, ASKED].map(|asked| replayed(git, &asked));
    let [outside, inside, no_branch] = [&TOPLEVEL[..], &TOPLEVEL, &NO_BRANCH].map(|asked| replayed(git, asked));
    let [foreign, noreply_email, unanswered, ran_out] = REPLAYED;
    let shown = |answer: &Output| String::from_utf8_lossy(&answer.stdout).into_owned();
    let root_line = [root.path().canonicalize().unwrap().as_os_str().as_bytes(), LINE_END.as_bytes()].concat();
    assert!(shown(&author).contains(foreign) && shown(&noreply).contains(noreply_email) && inside.stdout == root_line);
    let real = Command::new(git).args(TOPLEVEL).current_dir(root.path()).env_remove(GIT_DIR).output().unwrap();
    let other_program = replayed(gh, &ASKED);
    assert!(
        outside.status.code() == real.status.code()
            && real.status.code() != Some(1)
            && [&author, &noreply, &inside].iter().all(|answer| answer.status.success())
            && [&exhausted, &no_branch, &other_program].iter().all(|answer| answer.status.code() == Some(1))
    );
    let complained = [&exhausted, &other_program].map(|answer| String::from_utf8_lossy(&answer.stderr).into_owned());
    assert!(complained[0].contains(ran_out) && complained[1].contains(unanswered), "{complained:?}");
}
