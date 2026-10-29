mod program_names;

use std::path::Path;

use syn::visit::Visit;

use program_names::ProgramNames;
use task::{Amount, Most, Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct GitStarters;

impl GitStarters {
    const PORT: &str = "crates/meta-git";

    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "only meta-git starts git",
            files
                .iter()
                .filter(|file| !file.path().starts_with(Path::new(Self::PORT)))
                .flat_map(|file| {
                    let mut names = ProgramNames::default();
                    names.visit_file(file.syntax());
                    names.lines().into_iter().map(move |line| {
                        Violation::new(
                            Site::Line(file.path().into(), line),
                            "names the git program outside meta-git; start git through the meta-git port",
                        )
                    })
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn amount(report: &Report) -> Amount {
        Amount::Count(Most::from(report.violations().len()))
    }
}

#[cfg(test)]
mod tests {
    use task::SourceFile;

    use super::GitStarters;

    const PORTED: &str = "crates/meta-git/src/lib.rs";
    const ELSEWHERE: &str = "crates/meta-log/src/git.rs";
    const SOURCE: &str = "const GIT: &str = \"git\";\nfn f() { Command::new(\"git\"); assert!(run(\"git\")); }\nfn g(w: &str) -> bool { w == \"git\" || \"gh\" != w }\n#[cfg(test)]\nmod tests { fn t() { Command::new(\"git\"); } }\n";
    const FLAGGED: [&str; 4] = [":1", ":2", ":2", ":5"];

    #[test]
    fn every_literal_naming_git_outside_meta_git_is_a_starter_and_a_comparison_is_not() {
        let file = |path: &str| SourceFile::parse(path, SOURCE.to_owned()).unwrap();
        let report = GitStarters::report(&[file(PORTED), file(ELSEWHERE)]);
        assert!(report.lists_exactly(&FLAGGED) && GitStarters::amount(&report).to_string() == "4");
    }
}
