mod declared;

use std::collections::{BTreeMap, BTreeSet};

use declared::Declared;
use syn::visit::Visit;
use syn::{Ident, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, ItemUnion, Type};
use task::{CrateName, SourceFile, Walk};

use super::fact_scan::FactScan;
use crate::field_kind::FieldKind;
use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;

#[derive(Debug, Default)]
pub struct Declarations {
    owner: Option<CrateName>,
    types: BTreeMap<Ident, BTreeSet<CrateName>>,
    traits: BTreeMap<Ident, BTreeSet<CrateName>>,
    enums: BTreeSet<Ident>,
}

impl Declarations {
    const FLAG: &'static str = "bool";
    const SLOT: &'static str = "Option";
    const SHARED: [&'static str; 6] = ["Mutex", "RwLock", "RefCell", "Cell", "OnceCell", "OnceLock"];

    #[must_use]
    pub fn scanned(files: &[&SourceFile]) -> Declarations {
        files.iter().fold(Declarations::default(), |declarations, file| declarations.within(file))
    }

    fn within(mut self, file: &SourceFile) -> Declarations {
        self.owner = file.crate_name().cloned();
        self.visit_file(file.syntax());
        self
    }

    #[must_use]
    pub fn type_key(&self, owner: &CrateName, ident: &Ident) -> TypeKey {
        Self::owning(self.types.get(ident), owner)
            .map_or_else(|| TypeKey::foreign(ident), |found| TypeKey::local(found, ident))
    }

    #[must_use]
    pub fn trait_key(&self, owner: &CrateName, ident: &Ident) -> TraitKey {
        Self::owning(self.traits.get(ident), owner)
            .map_or_else(|| TraitKey::foreign(ident), |found| TraitKey::local(found, ident))
    }

    #[must_use]
    pub fn declares_type(&self, ident: &Ident) -> bool {
        self.types.contains_key(ident)
    }

    #[must_use]
    pub fn local_types(&self) -> BTreeSet<TypeKey> {
        self.types
            .iter()
            .flat_map(|(ident, owners)| owners.iter().map(move |owner| TypeKey::local(owner, ident)))
            .collect()
    }

    fn owning<'set>(owners: Option<&'set BTreeSet<CrateName>>, owner: &'set CrateName) -> Option<&'set CrateName> {
        owners.and_then(|owners| owners.get(owner).or_else(|| Self::sole(owners)))
    }

    #[must_use]
    pub fn sole(owners: &BTreeSet<CrateName>) -> Option<&CrateName> {
        Some(owners).filter(|owners| owners.len() == 1).and_then(BTreeSet::first)
    }

    #[must_use]
    pub fn field_kind(&self, ty: &Type) -> FieldKind {
        match FactScan::named(ty) {
            Some(ident) if ident == Self::FLAG => FieldKind::Flag,
            Some(ident) if ident == Self::SLOT => FieldKind::Slot,
            Some(ident) if Self::SHARED.iter().any(|shared| ident == shared) => FieldKind::Shared,
            Some(ident) if self.enums.contains(ident) => FieldKind::Mode,
            _ => FieldKind::Plain,
        }
    }

    fn declare(&mut self, ident: &Ident, kind: Declared) {
        let table = match kind {
            Declared::Type | Declared::Enum => &mut self.types,
            Declared::Trait => &mut self.traits,
        };
        table.entry(ident.clone()).or_default().extend(self.owner.clone());
        self.enums.extend(matches!(kind, Declared::Enum).then(|| ident.clone()));
    }
}

impl<'ast> Visit<'ast> for Declarations {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_fn(&mut self, _: &'ast ItemFn) {}

    fn visit_item_impl(&mut self, _: &'ast ItemImpl) {}

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.declare(&item.ident, Declared::Type);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.declare(&item.ident, Declared::Enum);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.declare(&item.ident, Declared::Type);
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.declare(&item.ident, Declared::Trait);
    }
}
