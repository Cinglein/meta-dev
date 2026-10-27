use std::path::PathBuf;

use super::Contents;
use crate::domains::Domain;
use crate::policy::SpecSettings;

const SPEC: &str = "package = \"spec\"\nfile = \"context/SPEC.md\"\n";
const ONE: &str = "# Spec\n- `rule-id` (a, b): words\n# Not yet converted: hand-written prose, 3 words\n\n- old prose\n";
const RULES: &str = "# Spec\n- `rule-id` (a, b): many many many words here\n";
const FILES: [(&str, &str); 3] = [
    ("context/SPEC.md", ONE),
    ("CLAUDE.md", RULES),
    ("README.md", "a b"),
];
const DOMAINS: [&str; 3] = [
    "paths = [\"*\"]", "paths = [\"context/*\", \"CLAUDE.md\"]", "crates = [\"*\"]",
];
const SPEC_PROSE: [&str; 3] = ["18", "16", "0"];
const ALL_WORDS: [&str; 3] = ["31", "29", "0"];

#[test]
fn words_in_a_domain_skip_the_spec_rule_records_and_marker_but_count_its_unconverted_prose() {
    let spec: SpecSettings = toml::from_str(SPEC).unwrap();
    let [ruled, plain] = [Some(&spec), None].map(|spec| {
        FILES
            .iter()
            .fold(Contents::default(), |mut contents, (path, text)| {
                contents.hold(PathBuf::from(path), text, spec);
                contents
            })
    });
    let domains = DOMAINS.map(|domain| toml::from_str::<Domain>(domain).unwrap());
    let spent = |contents: &Contents| {
        domains
            .each_ref()
            .map(|domain| contents.words(domain).to_string())
    };
    assert_eq!(spent(&ruled), SPEC_PROSE);
    assert_eq!(spent(&plain), ALL_WORDS);
}
