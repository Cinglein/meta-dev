use super::Project;

const REMOTE: &str = "origin";
const REFUSED: [&str; 2] = ["[ci]\nstop_millis = 9000", "[ci]\ngit_millis = \"soon\""];
const NAMED: [&str; 2] = ["stop_millis must be under", "git_millis"];

#[test]
fn a_project_serves_the_config_read_from_its_meta_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("meta.toml"), format!("remote = \"{REMOTE}\"")).unwrap();
    let project = Project::try_from(dir.path()).unwrap();
    assert_eq!(project.config().remote().unwrap().to_string(), REMOTE);
}

#[test]
fn a_refused_ci_setting_refuses_the_project_with_its_own_message() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let refused = REFUSED.map(|text| {
        std::fs::write(dir.path().join("meta.toml"), text).unwrap();
        Project::try_from(dir.path()).err().map(|failure| failure.to_string()).unwrap_or_default()
    });
    assert!(refused.iter().zip(NAMED).all(|(refused, named)| refused.contains(named)), "{refused:?}");
}
