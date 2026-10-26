mod finding;

use std::collections::BTreeSet;

use finding::Finding;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, Item, ItemEnum, ItemImpl, ReturnType, Type};

use task::{LineNumber, Name, Report, Site, SourceFile, Violation};

#[derive(Debug, Default)]
pub struct EnumPredicates {
    enums: BTreeSet<Name>,
}

impl EnumPredicates {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        let mut predicates = EnumPredicates::default();
        files
            .iter()
            .for_each(|file| predicates.visit_file(file.syntax()));
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
            .filter(|block| self.is_enum(&block.self_ty))
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

    fn is_enum(&self, implemented: &Type) -> bool {
        match implemented {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|segment| self.enums.contains(&Name::from(&segment.ident))),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for EnumPredicates {
    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.enums.insert(Name::from(&item.ident));
    }
}

#[cfg(test)]
mod tests {
    use super::EnumPredicates;
    use task::SourceFile;
    const SOURCE: &str = "
pub enum Zoom { Day, Leg }
impl Zoom {
    pub fn is_leg(self) -> bool { matches!(self, Zoom::Leg) }
    pub fn other(self) -> Zoom { self }
}
impl PartialEq for Zoom { fn eq(&self, other: &Zoom) -> bool { true } }
pub struct Plain;
impl Plain { pub fn is_plain(&self) -> bool { true } }
";
    const REPORTED: [&str; 1] = [":4: fn is_leg -> bool"];

    #[test]
    fn flags_inherent_enum_methods_returning_bool_only() {
        let file = SourceFile::parse("zoom.rs", SOURCE.to_owned()).unwrap();
        let report = EnumPredicates::report(core::slice::from_ref(&file));
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
