mod ancestor;
mod inside_sibling;
mod relation;
mod same;

pub use ancestor::Ancestor;
pub use inside_sibling::InsideSibling;
pub use relation::Relation;
pub use same::Same;

use std::fmt;
use std::path::Path;

use task::UsePath;
use task::{Name, Whole};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath {
    crate_root: String,
    segments: Vec<Name>,
    file: String,
}

impl ModulePath {
    const SOURCE_ROOTS: [&str; 2] = ["src/lib.rs", "src/main.rs"];
    const CRATE: &str = "crate";
    const SUPER: &str = "super";
    const ANCHORS: [&str; 3] = ["crate", "super", "self"];

    pub fn of_file(path: &Path) -> Option<ModulePath> {
        let file = path.to_string_lossy();
        let (crate_root, inside) = file.split_once("/src/")?;
        let inside = Path::new(inside);
        let stem = inside.file_stem()?.to_string_lossy().into_owned();
        let segments = inside
            .parent()
            .into_iter()
            .flat_map(Path::components)
            .map(|component| Name::from(&*component.as_os_str().to_string_lossy()))
            .chain((!Self::is_crate_root(&file) && stem != "mod").then(|| Name::from(&*stem)))
            .collect();
        Some(ModulePath {
            crate_root: crate_root.to_owned(),
            segments,
            file: file.into_owned(),
        })
    }

    pub fn from_use(
        file: &ModulePath,
        path: &UsePath,
        modules: &[ModulePath],
    ) -> Option<ModulePath> {
        let first = path.first()?;
        let start = if *first == Self::CRATE {
            Vec::new()
        } else if *first == Self::SUPER {
            file.parent()?.segments
        } else {
            file.segments.clone()
        };
        let skip = usize::from(first.matches::<Whole>(&Self::ANCHORS));
        (skip != 0
            || modules
                .iter()
                .any(|module| module.relates::<Same>(&file.child(first))))
        .then_some(())?;
        let segments =
            path.segments()
                .iter()
                .skip(skip)
                .try_fold(start, |mut segments, segment| {
                    if *segment == Self::SUPER {
                        segments.pop().and(Some(segments))
                    } else {
                        segments.push(segment.clone());
                        Some(segments)
                    }
                })?;
        Some(ModulePath {
            crate_root: file.crate_root.clone(),
            segments,
            file: String::new(),
        })
    }

    pub fn deepest_module_containing<'tree>(
        modules: &'tree [ModulePath],
        target: &ModulePath,
        user: &ModulePath,
    ) -> Option<&'tree ModulePath> {
        modules
            .iter()
            .filter(|module| module.relates::<Ancestor>(target) && !module.relates::<Same>(user))
            .max_by_key(|module| module.segments.len())
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn segments(&self) -> &[Name] {
        &self.segments
    }

    pub fn parent(&self) -> Option<ModulePath> {
        let (_, parents) = self.segments.split_last()?;
        Some(ModulePath {
            crate_root: self.crate_root.clone(),
            segments: parents.to_vec(),
            file: String::new(),
        })
    }

    pub fn relates<R: Relation>(&self, other: &ModulePath) -> bool {
        R::holds(self, other)
    }

    pub fn crosses_directory_boundary(&self, user: &ModulePath, modules: &[ModulePath]) -> bool {
        (1..self.segments.len()).any(|cut| {
            let boundary = self.truncated_to_length(cut);
            modules
                .iter()
                .any(|module| module.relates::<Same>(&boundary) && module.file.ends_with("/mod.rs"))
                && !boundary.relates::<Ancestor>(user)
        })
    }

    pub fn truncated_to_length(&self, length: usize) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().take(length).cloned().collect(),
            file: String::new(),
        }
    }

    fn child(&self, name: &Name) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self
                .segments
                .iter()
                .cloned()
                .chain(Some(name.clone()))
                .collect(),
            file: String::new(),
        }
    }

    fn is_crate_root(file: &str) -> bool {
        Self::SOURCE_ROOTS.iter().any(|root| file.ends_with(root))
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments
            .iter()
            .try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}
