mod declarations;
mod fact_scan;

use std::path::PathBuf;

use declarations::Declarations;
use fact_scan::FactScan;
use task::SourceFile;
use task::SupportModules;

use crate::fact_source::FactSource;
use crate::facts::Facts;

#[derive(Debug)]
pub struct SynSource<'files> {
    files: &'files [SourceFile],
}

impl<'files> SynSource<'files> {
    #[must_use]
    pub fn new(files: &'files [SourceFile]) -> SynSource<'files> {
        SynSource { files }
    }
}

impl FactSource for SynSource<'_> {
    fn facts(&self) -> Facts {
        let support: Vec<PathBuf> = self.files.iter().flat_map(SupportModules::paths).collect();
        let production: Vec<&SourceFile> =
            self.files.iter().filter(|file| !support.contains(&file.path().to_path_buf())).collect();
        let declarations = Declarations::scanned(&production);
        FactScan::scanned(&declarations, &production)
    }
}
