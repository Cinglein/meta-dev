use meta_door::Door;

use super::Project;
use crate::request::tests::Repository;

const REMOTE: &str = "origin";

#[test]
fn a_project_serves_the_config_read_from_its_meta_toml() {
    let dir = Repository::new();
    Door::write(dir.path().join("meta.toml"), format!("remote = \"{REMOTE}\"")).unwrap();
    let project = Project::try_from(dir.path()).unwrap();
    assert_eq!(project.config().remote().unwrap().to_string(), REMOTE);
}
