use std::path::{Component, Path};

use strum::{Display, EnumString};

use task::LineNumber;
use task::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum Scope {
    Line,
    File,
    Crate,
    Workspace,
}

impl Scope {
    const SOURCES: &str = "src";

    pub fn site(self, path: &Path, line: LineNumber) -> Site {
        match self {
            Scope::Line => Site::Line(path.to_path_buf(), line),
            Scope::File => Site::File(path.to_path_buf()),
            Scope::Crate => Site::File(
                path.components()
                    .take_while(|component| component.as_os_str() != Self::SOURCES)
                    .map(Component::as_os_str)
                    .collect(),
            ),
            Scope::Workspace => Site::Workspace,
        }
    }
}
