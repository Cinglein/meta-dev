mod finding;
mod module_path;

use std::collections::{BTreeMap, BTreeSet};

use finding::Finding;
use module_path::{Ancestor, InsideSibling, ModulePath};

use task::ReexportCount;
use task::Site;
use task::SourceFile;
use task::UseEdges;
use task::UsePath;
use task::Violation;
use task::{Limit, Policy};

pub struct ModuleTree {
    modules: Vec<ModulePath>,
    users: BTreeMap<ModulePath, BTreeSet<ModulePath>>,
    deep_uses: Vec<Violation>,
    reexports: BTreeMap<ModulePath, ReexportCount>,
}

impl ModuleTree {
    pub fn collect(files: &[SourceFile]) -> ModuleTree {
        let empty = ModuleTree {
            modules: files
                .iter()
                .filter_map(|file| ModulePath::of_file(file.path()))
                .collect(),
            users: BTreeMap::new(),
            deep_uses: Vec::new(),
            reexports: BTreeMap::new(),
        };
        files.iter().fold(empty, ModuleTree::absorbed)
    }

    pub fn violations(&self) -> impl Iterator<Item = Violation> + '_ {
        let deep_uses = self.deep_uses.iter().cloned();
        self.modules
            .iter()
            .filter_map(|module| {
                self.home_of(module).map(|home| {
                    Violation::new(
                        Site::File(module.file().into()),
                        Finding::UsedFromOneSibling { home },
                    )
                })
            })
            .chain(deep_uses)
    }

    pub fn crowded<'tree>(
        &'tree self,
        policy: &'tree Policy,
    ) -> impl Iterator<Item = Violation> + 'tree {
        let most = policy.limit(Limit::Reexports);
        self.reexports
            .iter()
            .filter(|(module, _)| !module.segments().is_empty())
            .filter(move |(_, count)| !most.allows(**count))
            .map(move |(module, count)| {
                Violation::new(
                    Site::File(module.file().into()),
                    Finding::CrowdedReexports {
                        count: *count,
                        most,
                    },
                )
            })
    }

    fn absorbed(mut self, file: &SourceFile) -> ModuleTree {
        match ModulePath::of_file(file.path()) {
            Some(module) => {
                self.reexports
                    .insert(module.clone(), UseEdges::count_reexports(file.syntax()));
                UseEdges::in_file(file.syntax())
                    .iter()
                    .fold(self, |tree, path| tree.absorbing(&module, path))
            }
            None => self,
        }
    }

    fn absorbing(mut self, module: &ModulePath, path: &UsePath) -> ModuleTree {
        match ModulePath::from_use(module, path, &self.modules) {
            Some(target) => {
                let used = ModulePath::deepest_module_containing(&self.modules, &target, module);
                self.record_user(used.cloned(), module);
                self.deep_uses.extend(
                    target
                        .crosses_directory_boundary(module, &self.modules)
                        .then(|| {
                            Violation::new(
                                Site::File(module.file().into()),
                                Finding::ReachesPastModRs {
                                    target: target.clone(),
                                },
                            )
                        }),
                );
                self
            }
            None => self,
        }
    }

    fn record_user(&mut self, used: Option<ModulePath>, module: &ModulePath) -> bool {
        match used {
            Some(used) => self.users.entry(used).or_default().insert(module.clone()),
            None => false,
        }
    }

    fn home_of(&self, module: &ModulePath) -> Option<ModulePath> {
        let outsiders: Vec<&ModulePath> = self
            .users
            .get(module)?
            .iter()
            .filter(|user| !module.relates::<Ancestor>(user))
            .collect();
        outsiders
            .iter()
            .all(|user| user.relates::<InsideSibling>(module))
            .then(|| {
                let homes: BTreeSet<ModulePath> = outsiders
                    .iter()
                    .map(|user| user.truncated_to_length(module.segments().len()))
                    .collect();
                (homes.len() == 1)
                    .then(|| homes.into_iter().next())
                    .flatten()
            })
            .flatten()
    }
}
