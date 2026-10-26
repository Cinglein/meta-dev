use crate::request::local::LocalFile;

const FILE: &str = "record.json";
const TEXT: &str = "{}";
const STAGED: &str = ".staged";
const RENAMING: &str = ": renaming to ";

#[test]
fn every_writer_stages_under_its_own_name_that_holds_the_process_id() {
    let file = LocalFile::new(std::env::temp_dir().join(FILE));
    let [first, second] = [file.staging(), file.staging()];
    assert_ne!(first, second);
    assert!(first.to_string_lossy().contains(&std::process::id().to_string()));
}

#[test]
fn a_failed_replacement_names_the_staging_path_and_the_target() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join(FILE);
    std::fs::create_dir(&target).unwrap();
    let failure = LocalFile::new(target.clone()).replaced(TEXT).unwrap_err();
    assert!(
        failure.to_string().contains(STAGED)
            && failure.to_string().contains(&format!("{RENAMING}{}", target.to_string_lossy()))
    );
}
