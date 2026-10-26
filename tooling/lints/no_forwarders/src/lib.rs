mod callee;
mod finding;
mod forwarder;
mod forwarders;
mod workspace_types;

use finding::Finding;
use forwarder::Forwarder;
use forwarders::Forwarders;
use syn::visit::Visit;
use workspace_types::WorkspaceTypes;

use task::LineNumber;
use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct NoForwarders;

impl NoForwarders {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let types = WorkspaceTypes::collect(files);
        Report::new(
            "methods do their own work, they do not forward to another type of ours",
            files
                .iter()
                .flat_map(|file| {
                    Self::forwarders(file, &types)
                        .into_iter()
                        .map(move |forwarder| {
                            Violation::new(
                                Site::Line(
                                    file.path().to_owned(),
                                    LineNumber::from(forwarder.name().span()),
                                ),
                                Finding::Forwards {
                                    name: Name::from(forwarder.name()),
                                    callee: forwarder.callee().clone(),
                                },
                            )
                        })
                })
                .collect(),
        )
    }

    fn forwarders(file: &SourceFile, types: &WorkspaceTypes) -> Vec<Forwarder> {
        let mut scan = Forwarders::new(types);
        scan.visit_file(file.syntax());
        scan.found()
    }
}
