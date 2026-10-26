#![allow(clippy::unwrap_used)]

mod rm_scene;

use rm_scene::{CARGO_TAG, COMMIT, CONFIG, CacheTag, Git, ORIGIN, Refusal, Scene};

const TAGGED: &str = "tagged";
const STRAYED: &str = "strayed";
const NOTES: &str = "notes";
const NESTED: &str = "nested";
const INNER_REPO: &str = "debug/inner/.git";
const HOSTING: &str = "hosting";
const TARGET: &str = "target";
const OTHER: &str = "other";
const GUEST: &str = "guest";
const HOLDS_REPO: &str = "cache tag over a git repo";
const TAG_HAZARDS: [Refusal; 4] = [
    Refusal::new(TAGGED, HOLDS_REPO),
    Refusal::new(STRAYED, "not cargo's layout"),
    Refusal::new(NESTED, HOLDS_REPO),
    Refusal::new(HOSTING, HOLDS_REPO),
];
const HIDDEN: &str = "hidden";
const SKIP: [&str; 3] = ["update-index", "--skip-worktree", "meta.toml"];
const GITLINKED: &str = "gitlinked";
const GITLINK: [&str; 4] =
    ["update-index", "--add", "--cacheinfo", "160000,1111111111111111111111111111111111111111,sub"];
const ABSORBED: &str = "absorbed";
const MODULES: &str = ".git/modules";
const TAG_ONLY: &str = "tag-only";
const DANGLING: [&str; 6] = ["commit-tree", "HEAD^{tree}", "-p", "HEAD", "-m", "tagged"];
const TAG: [&str; 2] = ["tag", "only-here"];
const LENDER: &str = "lender";
const BORROWER: &str = "borrower";
const SUBMODULES: &str = "has submodules";
const GIT_HAZARDS: [Refusal; 5] = [
    Refusal::new(HIDDEN, "index flags set"),
    Refusal::new(GITLINKED, SUBMODULES),
    Refusal::new(ABSORBED, SUBMODULES),
    Refusal::new(TAG_ONLY, "commits not on a remote"),
    Refusal::new(LENDER, "objects borrowed by another repo"),
];

#[test]
fn cache_tags_never_cover_a_repo_or_foreign_files() {
    let scene = Scene::new();
    Git::run(&CacheTag::plant(scene.cloned(ORIGIN, TAGGED), CARGO_TAG), &COMMIT);
    std::fs::write(CacheTag::plant(scene.place(STRAYED), CARGO_TAG).join(NOTES), CONFIG).unwrap();
    std::fs::create_dir_all(CacheTag::plant(scene.place(NESTED), CARGO_TAG).join(INNER_REPO)).unwrap();
    let hosting = CacheTag::plant(Git::worktree(&scene.project(), scene.place(HOSTING)).join(TARGET), CARGO_TAG);
    Git::worktree(&scene.cloned(ORIGIN, OTHER), hosting.join(GUEST));
    assert_eq!(scene.keeps(&TAG_HAZARDS), Ok(()));
}

#[test]
fn index_flags_submodules_tags_and_borrowed_objects_are_kept() {
    let scene = Scene::new();
    let hidden = Git::worktree(&scene.project(), scene.place(HIDDEN));
    Git::run(&hidden, &SKIP);
    std::fs::write(hidden.join("meta.toml"), NOTES).unwrap();
    Git::run(&Git::worktree(&scene.project(), scene.place(GITLINKED)), &GITLINK);
    std::fs::create_dir(scene.cloned(ORIGIN, ABSORBED).join(MODULES)).unwrap();
    let dangling = Git::run(&scene.project(), &DANGLING).transcript();
    Git::run(&scene.project(), &[&TAG[..], &[dangling.lines().next().unwrap()]].concat());
    Git::worktree(&scene.project(), scene.place(TAG_ONLY));
    scene.cloned(&scene.cloned(ORIGIN, LENDER).to_string_lossy(), BORROWER);
    assert_eq!(scene.keeps(&GIT_HAZARDS), Ok(()));
}
