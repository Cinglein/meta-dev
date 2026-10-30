use elegance::{EleganceSettings, Measures, Scoring, SmellInventory, SynSource};
use meta_golden::Golden;
use settings::EmbeddedDefaults;
use task::Workspace;

use crate::task::failure::Failure;

pub struct EleganceReport;

impl EleganceReport {
    const GOLDEN: &'static str = "tooling/elegance/goldens/meta-dev.txt";
    const CI: &'static str = "CI";

    pub fn text(workspace: &Workspace) -> Result<String, Failure> {
        let files = workspace.source_files()?;
        let measures = Measures::measured(&SynSource::new(&files), &EleganceSettings::defaults());
        Ok(format!("{measures}{}", Scoring::new(&measures, &SmellInventory::defaults())))
    }

    pub fn golden(workspace: &Workspace) -> Result<(), Failure> {
        let golden = Golden::new(&workspace.root().join(Self::GOLDEN));
        let report = Self::text(workspace)?;
        Ok(match std::env::var_os(Self::CI) {
            Some(_) => golden.checked(&report)?,
            None => golden.written(&report)?,
        })
    }
}
