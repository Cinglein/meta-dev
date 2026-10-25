mod module_path;
mod nesting_message;
mod reexport_limit;
mod use_edge;

use std::collections::{BTreeMap, BTreeSet};

use module_path::ModulePath;
use nesting_message::NestingMessage;
use reexport_limit::ReexportLimit;
use use_edge::UseEdges;

use crate::count::Count;
use crate::rule::name::Name;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct ModuleTree {
    modules: Vec<ModulePath>,
    users: BTreeMap<ModulePath, BTreeSet<ModulePath>>,
    deep_uses: Vec<Violation>,
    reexports: BTreeMap<ModulePath, Count>,
}

impl ModuleTree {
    pub fn collect(files: &[SourceFile]) -> ModuleTree {
        let empty = ModuleTree {
            modules: files.iter().filter_map(|file| ModulePath::of_file(file.path())).collect(),
            users: BTreeMap::new(),
            deep_uses: Vec::new(),
            reexports: BTreeMap::new(),
        };
        files.iter().fold(empty, ModuleTree::absorb)
    }

    pub fn misplaced(&self) -> impl Iterator<Item = Violation> + '_ {
        self.modules.iter().filter_map(|module| {
            self.home_of(module)
                .map(|home| Violation::new(Site::File(module.file().to_owned()), NestingMessage::Misplaced(home)))
        })
    }

    pub fn deep_uses(&self) -> impl Iterator<Item = Violation> + '_ {
        self.deep_uses.iter().cloned()
    }

    pub fn crowded(&self) -> impl Iterator<Item = Violation> + '_ {
        self.reexports
            .iter()
            .filter(|(module, _)| !module.segments().is_empty())
            .filter(|(_, count)| ReexportLimit::MAX.exceeded_by(**count))
            .map(|(module, count)| {
                Violation::new(Site::File(module.file().to_owned()), NestingMessage::Crowded(*count))
            })
    }

    fn absorb(mut self, file: &SourceFile) -> ModuleTree {
        match ModulePath::of_file(file.path()) {
            Some(module) => {
                self.reexports.insert(module.clone(), UseEdges::count_reexports(file.syntax()));
                UseEdges::in_file(file.syntax()).iter().fold(self, |tree, path| tree.absorb_use(&module, path))
            }
            None => self,
        }
    }

    fn absorb_use(mut self, module: &ModulePath, path: &[Name]) -> ModuleTree {
        let target = ModulePath::from_use(module, path, &self.modules);
        let used = target
            .as_ref()
            .and_then(|target| ModulePath::deepest_module_containing(&self.modules, target, module))
            .cloned();
        used.into_iter().for_each(|used| {
            self.users.entry(used).or_default().insert(module.clone());
        });
        let deep = target
            .filter(|target| target.crosses_directory_boundary(module, &self.modules))
            .map(|target| Violation::new(Site::File(module.file().to_owned()), NestingMessage::DeepUse(target)));
        self.deep_uses.extend(deep);
        self
    }

    fn home_of(&self, module: &ModulePath) -> Option<ModulePath> {
        let outsiders: Vec<&ModulePath> =
            self.users.get(module)?.iter().filter(|user| !module.is_ancestor_of(user)).collect();
        let homes: BTreeSet<ModulePath> =
            outsiders.iter().map(|user| user.truncated_to_len(module.segments().len())).collect();
        (outsiders.iter().all(|user| user.is_inside_a_sibling_of(module)) && homes.len() == 1)
            .then(|| homes.into_iter().next())
            .flatten()
    }
}
