mod file_sites;
mod finding;
mod number_sites;
mod text_sites;

use std::path::PathBuf;

use file_sites::FileSites;
use number_sites::NumberSites;
use text_sites::TextSites;

use task::Policy;
use task::Report;
use task::SourceFile;
use task::SupportModules;

#[derive(Debug)]
pub struct PrimitiveBoundary;

impl PrimitiveBoundary {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        let support: Vec<PathBuf> = files.iter().flat_map(SupportModules::paths).collect();
        Report::new(
            "primitives appear only at a type's boundary; files only through the file door",
            files
                .iter()
                .filter(|file| !support.contains(&file.path().to_owned()))
                .flat_map(|file| {
                    NumberSites::violations(file, policy)
                        .into_iter()
                        .chain(TextSites::violations(file))
                })
                .chain(
                    files
                        .iter()
                        .filter(|file| !policy.file_door().covers(file.path()))
                        .flat_map(FileSites::violations),
                )
                .collect(),
        )
    }
}
