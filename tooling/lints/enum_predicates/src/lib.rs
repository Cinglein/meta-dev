mod finding;

use std::collections::{BTreeMap, BTreeSet};

use finding::Finding;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, Item, ItemEnum, ItemImpl, ReturnType, Type};

use task::{CrateName, LineNumber, Name, Report, Site, SourceFile, Violation};

#[derive(Debug, Default)]
pub struct EnumPredicates {
    current: Option<CrateName>,
    enums: BTreeMap<Option<CrateName>, BTreeSet<Name>>,
}

impl EnumPredicates {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let predicates = files
            .iter()
            .fold(EnumPredicates::default(), |mut predicates, file| {
                predicates.enter(file);
                predicates
            });
        Report::new(
            "an enum method returning bool throws away the variant; return an enum or match where the answer is used",
            files
                .iter()
                .flat_map(|file| predicates.violations(file))
                .collect(),
        )
    }

    fn violations(&self, file: &SourceFile) -> Vec<Violation> {
        file.syntax()
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Impl(block) if block.trait_.is_none() => Some(block),
                _ => None,
            })
            .filter(|block| self.is_enum(file.crate_name(), &block.self_ty))
            .flat_map(|block| Self::predicates(file, block))
            .collect()
    }

    fn predicates(file: &SourceFile, block: &ItemImpl) -> Vec<Violation> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(&function.sig),
                _ => None,
            })
            .filter(|signature| match &signature.output {
                ReturnType::Type(_, returned) => {
                    matches!(returned.as_ref(), Type::Path(path) if path.path.is_ident("bool"))
                }
                ReturnType::Default => false,
            })
            .map(|signature| {
                Violation::new(
                    Site::Line(file.path().into(), LineNumber::from(signature.span())),
                    Finding::BoolPredicate {
                        name: Name::from(&signature.ident),
                    },
                )
            })
            .collect()
    }

    fn is_enum(&self, owner: Option<&CrateName>, implemented: &Type) -> bool {
        let named = match implemented {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| Name::from(&segment.ident)),
            _ => None,
        };
        named
            .zip(self.enums.get(&owner.cloned()))
            .is_some_and(|(name, enums)| enums.contains(&name))
    }

    fn enter(&mut self, file: &SourceFile) {
        self.current = file.crate_name().cloned();
        self.visit_file(file.syntax());
    }
}

impl<'ast> Visit<'ast> for EnumPredicates {
    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.enums
            .entry(self.current.clone())
            .or_default()
            .insert(Name::from(&item.ident));
    }
}
