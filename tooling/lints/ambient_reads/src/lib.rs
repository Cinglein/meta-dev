mod ambient_paths;

use std::path::Path;

use ambient_paths::AmbientPaths;
use task::{Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct AmbientReads;

impl AmbientReads {
    const SCOPE: &str = "crates/meta-cli/src";
    const ENTRIES: [&str; 2] = [
        "crates/meta-cli/src/main.rs", "crates/meta-cli/src/gh_replay.rs",
    ];
    const HEADLINE: &str = "only main reads the process environment";
    const FINDING: &str = "reads the process environment outside main; take it from the Call";

    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let entry = |path: &Path| Self::ENTRIES.iter().any(|entry| path == Path::new(entry));
        let scoped = files
            .iter()
            .filter(|file| file.path().starts_with(Self::SCOPE) && !entry(file.path()));
        let found = scoped.flat_map(|file| {
            AmbientPaths::found(file).into_iter().map(move |line| {
                Violation::new(Site::Line(file.path().into(), line), Self::FINDING)
            })
        });
        Report::new(Self::HEADLINE, found.collect())
    }
}

#[cfg(test)]
mod tests {
    use task::SourceFile;

    use super::AmbientReads;

    const LIBRARY: &str = "crates/meta-cli/src/request.rs";
    const ENTRY: &str = "crates/meta-cli/src/main.rs";
    const ELSEWHERE: &str = "crates/meta-run/src/run.rs";
    const SOURCE: &str = "use std::env;\nfn f() { std::env::var(X); }\nfn g() { env::current_dir(); std::io::stdin(); }\nfn h() { let t = std::env::temp_dir(); env::var_os(Y).map(std::env::current_exe); }\n";
    const FLAGGED: [&str; 6] = [":2", ":3", ":3", ":4", ":4", ":4"];

    #[test]
    fn each_environment_read_in_the_cli_library_is_a_finding_and_main_or_another_crate_is_not() {
        let file = |path: &str| SourceFile::parse(path, SOURCE.to_owned()).unwrap();
        let report = AmbientReads::report(&[file(LIBRARY), file(ENTRY), file(ELSEWHERE)]);
        assert!(report.lists_exactly(&FLAGGED));
    }

    #[test]
    fn near_misses_of_the_environment_paths_are_not_findings() {
        let source = "fn f() { my::env::other(); env::set_var(A, B); io::stdout(); env::stdin(); io::var(); other::stdin(); other::var(); var(); stdin(); }\n";
        let file = SourceFile::parse(LIBRARY, source.to_owned()).unwrap();
        assert!(AmbientReads::report(&[file]).lists_exactly(&[]));
    }
}
