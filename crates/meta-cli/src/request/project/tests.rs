use super::Project;

const CONFIGS: [&str; 3] = ["remote = \"origin\"", "[ci]\nstop_millis = 9000", "[ci]\ngit_millis = \"soon\""];
const SHOWN: [&str; 3] = ["origin", "stop_millis must be under", "git_millis"];

#[test]
fn a_project_serves_its_meta_toml_a_refused_ci_setting_refuses_it_and_no_meta_toml_is_no_project() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    assert!(Project::found(dir.path()).is_ok_and(|project| project.is_none()));
    let shown = CONFIGS.map(|text| {
        std::fs::write(dir.path().join("meta.toml"), text).unwrap();
        Project::try_from(dir.path())
            .map_or_else(|failure| failure.to_string(), |project| format!("{:?}", project.config().remote()))
    });
    assert!(shown.iter().zip(SHOWN).all(|(shown, named)| shown.contains(named)), "{shown:?}");
}
