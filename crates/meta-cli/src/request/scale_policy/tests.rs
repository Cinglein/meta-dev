use meta_severity::Scales;

use super::ScalePolicy;

const POLICY: &str = "[[scales.\"features-and-maintenance/nav\"]]\nname = \"lost\"\nmeaning = \"no map\"\nparent = \"critical\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"slow\"\nmeaning = \"late\"\nparent = \"major\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"typo\"\nmeaning = \"text\"\nparent = \"minor\"\n";
const NAV: &str = "features-and-maintenance/nav";
const UNMAPPED: &str = "maps onto no level";

#[test]
fn scales_are_read_from_policy_and_are_the_defaults_without_one() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(ScalePolicy::read(dir.path()).unwrap(), Scales::default());
    std::fs::create_dir(dir.path().join("tooling")).unwrap();
    std::fs::write(dir.path().join(ScalePolicy::FILE), POLICY).unwrap();
    let nav = NAV.parse().unwrap();
    assert_eq!(ScalePolicy::read(dir.path()).unwrap().owner(&nav), Some(&nav));
    std::fs::write(dir.path().join(ScalePolicy::FILE), POLICY.replace("critical", "nope")).unwrap();
    assert!(ScalePolicy::read(dir.path()).unwrap_err().to_string().contains(UNMAPPED));
}
