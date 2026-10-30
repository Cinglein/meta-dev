mod command_paths;
mod program_names;

use std::path::Path;

use command_paths::CommandPaths;
use program_names::ProgramNames;
use task::{Amount, LineNumber, Most, Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct Starters {
    ports: &'static [&'static str],
    headline: &'static str,
    finding: &'static str,
    found: fn(&SourceFile) -> Vec<LineNumber>,
}

impl Starters {
    pub const GIT: Starters = Starters {
        ports: &["crates/meta-git"],
        headline: "only meta-git starts git",
        finding: "names the git program outside meta-git; start git through the meta-git port",
        found: ProgramNames::found,
    };
    pub const PROCESS: Starters = Starters {
        ports: &["crates/meta-run", "tooling/task"],
        headline: "only the runner crates start processes",
        finding: "names std::process::Command outside meta-run and task; start processes through their runners",
        found: CommandPaths::found,
    };

    #[must_use]
    pub fn report(&self, files: &[SourceFile]) -> Report {
        let outside = files.iter().filter(|file| {
            !self
                .ports
                .iter()
                .any(|port| file.path().starts_with(Path::new(port)))
        });
        Report::new(
            self.headline,
            outside
                .flat_map(|file| {
                    (self.found)(file).into_iter().map(move |line| {
                        Violation::new(Site::Line(file.path().into(), line), self.finding)
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

    use super::Starters;

    const PORTED: &str = "crates/meta-git/src/lib.rs";
    const ELSEWHERE: &str = "crates/meta-log/src/git.rs";
    const SOURCE: &str = "const GIT: &str = \"git\";\nfn f() { Command::new(\"git\"); assert!(run(\"git\")); }\nfn g(w: &str) -> bool { w == \"git\" || run(\"git\") }\n#[cfg(test)]\nmod tests { fn t() { Command::new(\"git\"); } }\n";
    const PORT_SOURCE: &str = "fn p() { Command::new(\"git\"); }\n";
    const FLAGGED: [&str; 5] = [":1", ":2", ":2", ":3", ":5"];
    const RUNNER: &str = "crates/meta-run/src/run.rs";
    const STARTS: &str = "use std::process::{Child, Command};\nuse std::{env, process::Command as C};\nfn f() { std::process::Command::new(X); Command::new(X); }\nuse std::process::Child;\n";
    const STARTED: [&str; 3] = [":1", ":2", ":3"];

    #[test]
    fn every_literal_naming_git_outside_meta_git_is_a_starter_and_a_comparison_is_not() {
        let file = |path: &str, source: &str| SourceFile::parse(path, source.to_owned()).unwrap();
        let report = Starters::GIT.report(&[file(PORTED, PORT_SOURCE), file(ELSEWHERE, SOURCE)]);
        assert!(report.lists_exactly(&FLAGGED) && Starters::amount(&report).to_string() == "5");
    }

    #[test]
    fn every_path_to_the_std_command_outside_the_runners_is_a_starter() {
        let file = |path: &str| SourceFile::parse(path, STARTS.to_owned()).unwrap();
        assert!(
            Starters::PROCESS
                .report(&[file(RUNNER), file(ELSEWHERE)])
                .lists_exactly(&STARTED)
        );
    }
}
