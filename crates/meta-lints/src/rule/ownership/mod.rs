mod ownership_rule;

pub use ownership_rule::OwnershipRule;

use crate::rule::report::Report;
use crate::source_file::SourceFile;

pub struct Ownership;

impl Ownership {
    pub fn report(files: &[SourceFile], rules: &[OwnershipRule]) -> Report {
        Report::new(
            "each owned token appears only under its owner",
            rules.iter().flat_map(|rule| files.iter().flat_map(|file| rule.violations(file))).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Ownership, OwnershipRule};
    use crate::source_file::SourceFile;

    const RULE: &str = "owner = \"crates/app/session/\"\nwithin = \"crates/app/\"\ntokens = [\"use_signal\"]";
    const VIEW: &str = "fn render() { let open = use_signal(|| false); }";
    const PATHS: [&str; 3] = ["crates/app/session/mod.rs", "crates/app/view.rs", "crates/other/view.rs"];

    #[test]
    fn tokens_only_under_their_owner() {
        let rule: OwnershipRule = toml::from_str(RULE).expect("valid rule");
        let files: Vec<SourceFile> = PATHS
            .iter()
            .map(|path| SourceFile::parse((*path).to_owned(), VIEW.to_owned()).expect("valid rust"))
            .collect();
        let report = Ownership::report(&files, &[rule]).to_string();
        assert_eq!(
            report.lines().skip(1).collect::<Vec<_>>(),
            ["crates/app/view.rs:1: use_signal outside crates/app/session/"]
        );
    }
}
