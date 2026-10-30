mod scope;

use std::fmt;

use affected::Changes;
use elegance::{EleganceSettings, Measures, Scoring, SmellInventory, SynSource};
use meta_golden::Golden;
use settings::EmbeddedDefaults;
use task::{Branch, Policy, SourceFile, Workspace};

use crate::task::failure::Failure;
use scope::Scope;

pub struct EleganceReport {
    measures: Measures,
    inventory: SmellInventory,
}

impl EleganceReport {
    const GOLDEN: &'static str = "tooling/elegance/goldens/meta-dev.txt";
    const CI: &'static str = "CI";

    #[must_use]
    pub fn from_files(files: &[SourceFile]) -> EleganceReport {
        EleganceReport {
            measures: Measures::measured(&SynSource::new(files), &EleganceSettings::defaults()),
            inventory: SmellInventory::defaults(),
        }
    }

    pub fn measured(workspace: &Workspace) -> Result<EleganceReport, Failure> {
        Ok(Self::from_files(&workspace.source_files()?))
    }

    pub fn golden(workspace: &Workspace) -> Result<(), Failure> {
        let policy = Policy::read(workspace.root())?;
        let scope = Scope::judged(&Changes::since_main_paths(workspace, policy.branch(Branch::Base))?);
        match scope {
            Scope::Skipped => {
                eprintln!("elegance golden skipped: {}", scope.reason());
                Ok(())
            }
            Scope::Compared => Self::measured(workspace)?.compared(&Golden::new(&workspace.root().join(Self::GOLDEN))),
        }
    }

    pub fn compared(&self, golden: &Golden) -> Result<(), Failure> {
        let report = self.to_string();
        match std::env::var_os(Self::CI) {
            Some(_) => golden.checked(&report),
            None => golden.clone().written(&report),
        }?;
        Ok(())
    }
}

impl fmt::Display for EleganceReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.measures, Scoring::new(&self.measures, &self.inventory))
    }
}

#[cfg(test)]
mod tests {
    use meta_golden::Golden;
    use task::SourceFile;
    use tempfile::TempDir;

    use super::EleganceReport;

    const SOURCE: &str = "pub struct Unit { a: A }";
    const STALE: &str = "stale";
    const FILE: &str = "golden.txt";
    const PATH: &str = "unit/src/lib.rs";
    const CRATE: &str = "unit";
    const SECTIONS: [&str; 2] = ["B13 types whose method-field graph splits", "smell inventory scoring"];

    struct Report;

    impl Report {
        fn small() -> EleganceReport {
            let file = SourceFile::parse(PATH, SOURCE.to_owned()).unwrap().in_crate(CRATE.parse().unwrap());
            EleganceReport::from_files(&[file])
        }
    }

    #[test]
    fn the_report_prints_the_measures_and_their_scoring() {
        let text = Report::small().to_string();
        assert!(SECTIONS.iter().all(|section| text.contains(section)), "{text}");
    }

    #[test]
    fn a_stale_golden_is_refused_in_ci_and_rewritten_elsewhere() {
        let directory = TempDir::new().unwrap();
        let golden = Golden::new(&directory.path().join(FILE));
        golden.clone().written(STALE).unwrap();
        let report = Report::small();
        let refused = report.compared(&golden).is_err();
        assert!(refused || golden.text().unwrap() == report.to_string());
    }
}
