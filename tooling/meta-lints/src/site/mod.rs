mod line_number;

use std::fmt;

use std::path::Path;

use proc_macro2::Span;

use line_number::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    Workspace,
    File(String),
    Line(String, LineNumber),
}

impl Site {
    const WORKSPACE: &str = "workspace";

    pub fn spanned(path: &str, span: Span) -> Site {
        Site::Line(path.to_owned(), LineNumber::starting(span))
    }

    pub fn of_path(path: &Path) -> Site {
        Site::File(path.display().to_string())
    }

    pub fn within(root: &Path, path: &Path) -> Site {
        Self::of_path(path.strip_prefix(root).unwrap_or(path))
    }

    pub fn file(&self) -> &str {
        match self {
            Site::Workspace => Self::WORKSPACE,
            Site::File(path) | Site::Line(path, _) => path,
        }
    }
}

impl fmt::Display for Site {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Site::Workspace => formatter.write_str(Self::WORKSPACE),
            Site::File(path) => formatter.write_str(path),
            Site::Line(path, line) => write!(formatter, "{path}:{line}"),
        }
    }
}
