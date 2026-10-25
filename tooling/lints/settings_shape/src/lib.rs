mod deserialized_struct;
mod keys;
mod schema_scan;

use std::collections::BTreeSet;

use deserialized_struct::DeserializedStruct;
use keys::Keys;
use schema_scan::SchemaScan;

use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct SettingsShape;

impl SettingsShape {
    pub fn report(files: &[SourceFile]) -> Report {
        let scans: Vec<SchemaScan> = files.iter().map(SchemaScan::in_file).collect();
        let structs: Vec<&DeserializedStruct> =
            scans.iter().flat_map(SchemaScan::structs).collect();
        let owned = Self::owned(
            scans.iter().flat_map(SchemaScan::seeds).cloned().collect(),
            &structs,
        );
        Report::new(
            "a settings struct refuses keys it does not know",
            structs
                .iter()
                .filter(|item| owned.contains(item.name()) && item.keys() == Keys::Open)
                .map(|item| {
                    Violation::new(
                        Site::Line(item.path().into(), item.line()),
                        "struct is read from a settings file we write but lacks #[serde(deny_unknown_fields)]; a misspelt key must be an error, not a silent default",
                    )
                })
                .collect(),
        )
    }

    fn owned(seeds: BTreeSet<Name>, structs: &[&DeserializedStruct]) -> BTreeSet<Name> {
        let grown: BTreeSet<Name> = structs
            .iter()
            .filter(|item| seeds.contains(item.name()))
            .flat_map(|item| item.field_types().iter().cloned())
            .chain(seeds.iter().cloned())
            .collect();
        if grown == seeds {
            seeds
        } else {
            Self::owned(grown, structs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsShape;
    use task::SourceFile;
    const SOURCE: &str = "
#[derive(Deserialize)] pub struct Settings { schedule: Schedule, rounds: Count, plain: Plain }
impl EmbeddedDefaults for Settings { const DEFAULTS: &str = \"\"; }
#[derive(Deserialize)] #[serde(default)] pub struct Schedule { batches: usize }
#[derive(Deserialize)] pub struct Count(usize); #[derive(Debug)] pub struct Plain { x: usize }
#[derive(Deserialize)] pub struct Metadata { packages: Vec<Package> }
#[derive(Deserialize)] #[serde(deny_unknown_fields)] pub struct Strict { rounds: usize }
impl EmbeddedDefaults for Strict { const DEFAULTS: &str = \"\"; }
";
    const REPORTED: [&str; 2] = ["x.rs:2: struct is read", "x.rs:4: struct is read"];

    #[test]
    fn flags_named_structs_reachable_from_embedded_defaults_that_do_not_deny_unknown_keys() {
        let file = SourceFile::parse("crates/a/src/x.rs", SOURCE.to_owned()).unwrap();
        let report = SettingsShape::report(&[file]);
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
