use std::fmt;
use std::path::Path;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath {
    crate_root: String,
    segments: Vec<String>,
    file: String,
}

impl ModulePath {
    const SOURCE_ROOTS: [&str; 2] = ["src/lib.rs", "src/main.rs"];

    pub fn of_file(file: &str) -> Option<ModulePath> {
        let (crate_root, inside) = file.split_once("/src/")?;
        let inside = Path::new(inside);
        let stem = inside.file_stem()?.to_string_lossy().into_owned();
        let segments = inside
            .parent()
            .into_iter()
            .flat_map(Path::components)
            .map(|component| component.as_os_str().to_string_lossy().into_owned())
            .chain((!Self::is_crate_root(file) && stem != "mod").then_some(stem))
            .collect();
        Some(ModulePath { crate_root: crate_root.to_owned(), segments, file: file.to_owned() })
    }

    pub fn from_use(file: &ModulePath, path: &[Name], modules: &[ModulePath]) -> Option<ModulePath> {
        let first = path.first()?.as_str();
        let start = match first {
            "crate" => Vec::new(),
            "super" => file.parent()?.segments,
            _ => file.segments.clone(),
        };
        let skip = usize::from(matches!(first, "crate" | "super" | "self"));
        (skip != 0 || modules.iter().any(|module| module.same_module(&file.child(first)))).then_some(())?;
        let segments = path.iter().skip(skip).try_fold(start, Self::followed)?;
        Some(ModulePath { crate_root: file.crate_root.clone(), segments, file: String::new() })
    }

    pub fn deepest_module_containing<'tree>(
        modules: &'tree [ModulePath],
        target: &ModulePath,
        user: &ModulePath,
    ) -> Option<&'tree ModulePath> {
        modules
            .iter()
            .filter(|module| module.is_ancestor_of(target) && !module.same_module(user))
            .max_by_key(|module| module.segments.len())
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn parent(&self) -> Option<ModulePath> {
        let (_, parents) = self.segments.split_last()?;
        Some(ModulePath { crate_root: self.crate_root.clone(), segments: parents.to_vec(), file: String::new() })
    }

    pub fn same_module(&self, other: &ModulePath) -> bool {
        self.crate_root == other.crate_root && self.segments == other.segments
    }

    pub fn is_ancestor_of(&self, other: &ModulePath) -> bool {
        self.crate_root == other.crate_root && other.segments.starts_with(&self.segments)
    }

    pub fn is_inside_a_sibling_of(&self, module: &ModulePath) -> bool {
        let depth = module.segments.len();
        module.parent().is_some_and(|parent| parent.is_ancestor_of(self))
            && self.segments.len() >= depth
            && !self.truncated_to_len(depth).same_module(module)
    }

    pub fn crosses_directory_boundary(&self, user: &ModulePath, modules: &[ModulePath]) -> bool {
        (1..self.segments.len()).any(|cut| {
            let boundary = self.truncated_to_len(cut);
            modules.iter().any(|module| module.same_module(&boundary) && module.file.ends_with("/mod.rs"))
                && !boundary.is_ancestor_of(user)
        })
    }

    pub fn truncated_to_len(&self, len: usize) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().take(len).cloned().collect(),
            file: String::new(),
        }
    }

    fn child(&self, name: &str) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().cloned().chain(Some(name.to_owned())).collect(),
            file: String::new(),
        }
    }

    fn followed(mut segments: Vec<String>, segment: &Name) -> Option<Vec<String>> {
        match segment.as_str() {
            "super" => segments.pop().map(|_| segments),
            other => {
                segments.push(other.to_owned());
                Some(segments)
            }
        }
    }

    fn is_crate_root(file: &str) -> bool {
        Self::SOURCE_ROOTS.iter().any(|root| file.ends_with(root))
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}
