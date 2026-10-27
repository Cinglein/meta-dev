use task::{CommandLine, Policy, Selection};

#[derive(Debug)]
pub(super) struct CargoBuild;

impl CargoBuild {
    pub(super) fn examples(line: CommandLine, _: &Selection, _: &Policy) -> CommandLine {
        line
    }
}
