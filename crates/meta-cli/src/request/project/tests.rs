use std::time::UNIX_EPOCH;

use meta_git::{LooseObject, Reads as _, Repository};

use super::Project;
use crate::request::ci::{Commit, StartMoment};

const CONFIGS: [&str; 3] = ["[ci]\nstop_millis = 9000", "[ci]\ngit_millis = \"soon\"", "remote = \"origin\""];
const SHOWN: [&str; 3] = ["stop_millis must be under", "git_millis", "origin"];
const GIT: [&str; 3] = [".git/refs", ".git/objects/f2", ".git/HEAD"];
const SHAS: [&str; 2] = ["f2e83b47f04f1aadaaf72b9d15074e933a513dc6", "9271c82b76058cb9022078bf58c641cfc6a5b0d7"];

#[test]
fn a_project_serves_its_meta_toml_and_reads_head_and_object_births_through_git_and_no_meta_toml_is_no_project() {
    let dir = tempfile::tempdir().unwrap();
    GIT[..2].iter().for_each(|made| std::fs::create_dir_all(dir.path().join(made)).unwrap());
    assert!(Project::found(dir.path()).is_ok_and(|project| project.is_none()));
    let shown = CONFIGS.map(|text| {
        std::fs::write(dir.path().join("meta.toml"), text).unwrap();
        Project::try_from(dir.path())
            .map_or_else(|failure| failure.to_string(), |project| format!("{:?}", project.config().remote()))
    });
    assert!(shown.iter().zip(SHOWN).all(|(shown, named)| shown.contains(named)), "{shown:?}");
    let [head, unborn] = SHAS.map(|sha| sha.parse::<Commit>().unwrap());
    std::fs::write(dir.path().join(GIT[2]), SHAS[0]).unwrap();
    let loose = LooseObject::from(head.object_id().unwrap());
    let object = Repository::open(dir.path()).and_then(|repository| repository.read(&loose)).unwrap();
    std::fs::write(&object, SHAS[0]).unwrap();
    let modified = std::fs::metadata(object).and_then(|meta| meta.modified()).unwrap().duration_since(UNIX_EPOCH);
    let starts = [0, modified.unwrap().as_nanos()].map(|nanos| nanos.to_string().parse::<StartMoment>().unwrap());
    let project = Project::try_from(dir.path()).unwrap();
    let judged = starts.map(|start| [&head, &unborn].map(|commit| project.born_after(commit, &start)));
    assert!(judged == [[true, false], [false, false]] && project.head() == Some(head), "{judged:?}");
}
