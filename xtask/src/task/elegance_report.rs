mod scope;

use std::fmt;

use affected::Changes;
use elegance::{EleganceSettings, Measures, Scoring, SmellInventory, SynSource};
use meta_golden::Golden;
use settings::EmbeddedDefaults;
use task::{Branch, Policy, Workspace};

use crate::task::failure::Failure;
use scope::Scope;

pub struct EleganceReport {
    measures: Measures,
    inventory: SmellInventory,
}

impl EleganceReport {
    const GOLDEN: &'static str = "tooling/elegance/goldens/meta-dev.txt";
    const CI: &'static str = "CI";

    pub fn measured(workspace: &Workspace) -> Result<EleganceReport, Failure> {
        let files = workspace.source_files()?;
        Ok(EleganceReport {
            measures: Measures::measured(&SynSource::new(&files), &EleganceSettings::defaults()),
            inventory: SmellInventory::defaults(),
        })
    }

    pub fn golden(workspace: &Workspace) -> Result<(), Failure> {
        let policy = Policy::read(workspace.root())?;
        let scope = Scope::judged(&Changes::since_main_paths(workspace, policy.branch(Branch::Base))?);
        match scope {
            Scope::Skipped => {
                eprintln!("elegance golden skipped: {}", scope.reason());
                Ok(())
            }
            Scope::Compared => Self::measured(workspace)?.compared(workspace),
        }
    }

    fn compared(&self, workspace: &Workspace) -> Result<(), Failure> {
        let golden = Golden::new(&workspace.root().join(Self::GOLDEN));
        let report = self.to_string();
        match std::env::var_os(Self::CI) {
            Some(_) => golden.checked(&report),
            None => golden.written(&report),
        }?;
        Ok(())
    }
}

impl fmt::Display for EleganceReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.measures, Scoring::new(&self.measures, &self.inventory))
    }
}
