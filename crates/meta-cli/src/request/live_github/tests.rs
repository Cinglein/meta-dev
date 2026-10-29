use super::LiveGithub;

#[test]
fn a_project_requires_the_workflows_its_meta_toml_names_and_none_by_default() {
    let required = |settings: &str| {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join("meta.toml"), settings).unwrap();
        let project = crate::request::project::Project::try_from(dir.path()).unwrap();
        LiveGithub::for_project(&project).workflows.iter().map(ToString::to_string).collect::<Vec<_>>()
    };
    assert_eq!(required("remote = \"origin\"\nrequired_workflows = [\"ci\", \"docs\"]"), ["ci", "docs"]);
    assert_eq!(required("remote = \"origin\""), Vec::<String>::new());
}
