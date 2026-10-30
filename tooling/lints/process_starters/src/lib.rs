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
        ports: &["crates/meta-run", "tooling/task/src/command_line"],
        headline: "only the runners start processes",
        finding: "starts a std::process::Command outside meta-run and task::command_line; start it through a runner",
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
    const RUNNERS: [&str; 2] = [
        "crates/meta-run/src/run.rs", "tooling/task/src/command_line/mod.rs",
    ];
    const BESIDE_THE_RUNNER: &str = "tooling/task/src/policy.rs";
    const STARTS: &str = "use std::process::{Child, Command as C};\nuse std::process::*;\nfn f(c: &Command, ran: Ran) { ran.status(); c.spawn(); }\nfn g() { let held = C::new(X); std::process::Command::new(X).arg(Y).output(); }\nfn h() { assert!(Command::new(X).status().is_ok(), \"{}\", Child::new()); }\n";
    const STARTED: [&str; 4] = [":4", ":4", ":5", ":6"];
    const NAMED: (&str, &str) = (
        "crates/meta-log/src/named.rs",
        "\nuse std::process::Command;\nfn f() {\n    Command::new(X)\n        .arg(Y)\n        .spawn();\n}\n",
    );
    const OTHER_MODULE: (&str, &str) = (
        "crates/meta-log/src/other.rs",
        "use other::Command as D;\nuse other::*;\nfn f() { D::new(X).status(); Command::new(X).spawn(); }\n",
    );

    #[test]
    fn every_literal_naming_git_outside_meta_git_is_a_starter_and_a_comparison_is_not() {
        let file = |path: &str, source: &str| SourceFile::parse(path, source.to_owned()).unwrap();
        let report = Starters::GIT.report(&[file(PORTED, PORT_SOURCE), file(ELSEWHERE, SOURCE)]);
        assert!(report.lists_exactly(&FLAGGED) && Starters::amount(&report).to_string() == "5");
    }

    #[test]
    fn each_std_command_started_outside_the_runners_is_a_starter_however_it_is_imported() {
        let file = |path: &str, source: &str| SourceFile::parse(path, source.to_owned()).unwrap();
        let ([meta_run, command_line], (other, elsewhere)) = (RUNNERS, OTHER_MODULE);
        let files = [
            file(meta_run, STARTS),
            file(command_line, STARTS),
            file(BESIDE_THE_RUNNER, STARTS),
            file(other, elsewhere),
            file(NAMED.0, NAMED.1),
        ];
        assert!(Starters::PROCESS.report(&files).lists_exactly(&STARTED));
    }
}
