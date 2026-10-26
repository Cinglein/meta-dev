mod export;
mod finding;
mod foreign_references;

use export::Export;
use finding::Finding;
use foreign_references::ForeignReferences;

use task::BinaryFiles;

use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct PublicSurface;

impl PublicSurface {
    pub fn report(files: &[SourceFile]) -> Report {
        let binaries = BinaryFiles::collect(files);
        let references = files
            .iter()
            .map(|file| ForeignReferences::in_file(file, &binaries))
            .fold(ForeignReferences::default(), ForeignReferences::absorb);
        Report::new(
            "a crate exports exactly what another crate uses",
            files
                .iter()
                .flat_map(|file| {
                    Export::in_root(file)
                        .into_iter()
                        .filter(|export| {
                            references.depends_on(export.crate_name()) && !references.names(export)
                        })
                        .map(move |export| {
                            Violation::new(
                                Site::Line(file.path().into(), export.line()),
                                Finding::UnnamedExport {
                                    name: export.name().clone(),
                                },
                            )
                        })
                })
                .collect(),
        )
    }
}
