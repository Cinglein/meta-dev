mod cargo_tool;
mod tool;

pub use cargo_tool::CargoTool;
pub use tool::Tool;

use std::collections::BTreeSet;
use std::fmt;

use crate::command_line::CargoArgument;
use crate::crate_name::CrateName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    Workspace,
    Crates(BTreeSet<CrateName>),
}

impl Selection {
    #[must_use]
    pub fn arguments<T: Tool>(&self) -> Vec<String> {
        match self {
            Selection::Workspace => T::whole_workspace()
                .map(|argument| argument.to_string())
                .into_iter()
                .collect(),
            Selection::Crates(crates) => crates
                .iter()
                .flat_map(|name| [CargoArgument::Package.to_string(), name.to_string()])
                .collect(),
        }
    }

    #[must_use]
    pub fn includes(&self, crate_name: &CrateName) -> bool {
        match self {
            Selection::Workspace => true,
            Selection::Crates(crates) => crates.contains(crate_name),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Selection::Workspace => false,
            Selection::Crates(crates) => crates.is_empty(),
        }
    }
}

impl fmt::Display for Selection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selection::Workspace => formatter.write_str("the whole workspace"),
            Selection::Crates(crates) if crates.is_empty() => formatter.write_str("no crate"),
            Selection::Crates(crates) => {
                write!(
                    formatter,
                    "{}",
                    crates
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CargoTool, Selection};
    use std::collections::BTreeSet;
    const EVAL: &str = "eval";
    const SEARCH: &str = "search";
    const BOARD: &str = "board";
    const PACKAGES: [&str; 4] = ["--package", EVAL, "--package", SEARCH];
    const WORKSPACE: [&str; 1] = ["--workspace"];
    const SHOWN: [&str; 3] = ["the whole workspace", "no crate", "eval, search"];

    #[test]
    fn a_crate_selection_names_each_package_and_the_workspace_names_itself() {
        let (whole, empty) = (Selection::Workspace, Selection::Crates(BTreeSet::new()));
        let crates = Selection::Crates(BTreeSet::from([
            SEARCH.parse().unwrap(),
            EVAL.parse().unwrap(),
        ]));
        assert_eq!(crates.arguments::<CargoTool>(), PACKAGES);
        assert_eq!(
            (
                whole.arguments::<CargoTool>(),
                [whole.to_string(), empty.to_string(), crates.to_string()]
            ),
            (
                WORKSPACE.map(String::from).to_vec(),
                SHOWN.map(String::from)
            )
        );
        assert!(
            crates.includes(&EVAL.parse().unwrap()) && !crates.includes(&BOARD.parse().unwrap())
        );
    }
}
