use std::fmt;

use elegance::{EleganceSettings, Measures, Scoring, SmellInventory, SynSource};
use meta_golden::Golden;
use settings::EmbeddedDefaults;
use task::Workspace;

use crate::task::failure::Failure;

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

    pub fn golden(&self, workspace: &Workspace) -> Result<(), Failure> {
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
