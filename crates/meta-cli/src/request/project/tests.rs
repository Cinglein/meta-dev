use std::path::Path;

use super::Project;

const REMOTE: &str = "origin";

#[test]
fn a_project_serves_the_config_read_from_its_meta_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("meta.toml"), format!("remote = \"{REMOTE}\"")).unwrap();
    let project = Project::try_from(dir.path() as &Path).unwrap();
    assert_eq!(project.config().remote().unwrap().to_string(), REMOTE);
}
