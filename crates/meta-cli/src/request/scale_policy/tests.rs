use meta_door::{Change, Door};
use meta_severity::Scales;

use super::ScalePolicy;

const POLICY: &str = "[domains.\"features-and-maintenance/tracker\"]\npaths = [\"x\"]\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"lost\"\nmeaning = \"no map\"\nparent = \"critical\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"slow\"\nmeaning = \"late\"\nparent = \"major\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"typo\"\nmeaning = \"text\"\nparent = \"minor\"\n";
const NAV: &str = "features-and-maintenance/nav";
const KNOWN: [&str; 3] = [NAV, "features-and-maintenance/tracker", "architecture-and-planning"];
const UNKNOWN: &str = "features-and-maintenance/nowhere";
const UNMAPPED: &str = "maps onto no level";

#[test]
fn scales_and_known_divisions_are_read_from_policy_and_are_the_defaults_without_one() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(ScalePolicy::read(dir.path()).unwrap().scales(), &Scales::default());
    Door::changed(dir.path().join("tooling"), Change::Dir).unwrap();
    Door::write(dir.path().join(ScalePolicy::FILE), POLICY).unwrap();
    let nav = NAV.parse().unwrap();
    let policy = ScalePolicy::read(dir.path()).unwrap();
    assert_eq!(policy.scales().owner(&nav), Some(&nav));
    assert!(KNOWN.iter().all(|known| policy.known(&known.parse().unwrap()).is_ok()));
    assert!(policy.known(&UNKNOWN.parse().unwrap()).is_err());
    Door::write(dir.path().join(ScalePolicy::FILE), POLICY.replace("critical", "nope")).unwrap();
    assert!(ScalePolicy::read(dir.path()).unwrap_err().to_string().contains(UNMAPPED));
}
