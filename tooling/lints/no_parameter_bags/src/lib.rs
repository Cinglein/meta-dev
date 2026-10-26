mod bag_candidates;
mod finding;
mod name_uses;

use bag_candidates::BagCandidates;
use finding::Finding;
use name_uses::NameUses;

use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct NoParameterBags;

impl NoParameterBags {
    pub fn report(files: &[SourceFile]) -> Report {
        let uses: Vec<NameUses> = files.iter().map(NameUses::in_file).collect();
        Report::new(
            "a struct with no logic of its own that one other file consumes is a parameter bag",
            files
                .iter()
                .enumerate()
                .flat_map(|(index, file)| {
                    let candidates = BagCandidates::in_file(file);
                    candidates
                        .candidates()
                        .filter(|candidate| {
                            Self::has_one_consumer_besides_index(&uses, index, candidate.name())
                        })
                        .map(|candidate| {
                            Violation::new(
                                Site::Line(file.path().into(), candidate.line()),
                                Finding::ParameterBag {
                                    name: candidate.name().clone(),
                                },
                            )
                        })
                        .collect::<Vec<Violation>>()
                })
                .collect(),
        )
    }

    fn has_one_consumer_besides_index(uses: &[NameUses], index: usize, name: &Name) -> bool {
        uses.iter()
            .enumerate()
            .filter(|(other, file_uses)| *other != index && file_uses.mentions(name))
            .count()
            == 1
    }
}
