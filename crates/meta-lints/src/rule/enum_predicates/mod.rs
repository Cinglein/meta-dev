mod bool_predicate;

use std::collections::BTreeSet;

use bool_predicate::BoolPredicate;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, ItemEnum, ItemImpl, ReturnType, Type};

use crate::rule::name::Name;
use crate::rule::report::Report;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct EnumPredicates {
    enums: BTreeSet<Name>,
}

impl EnumPredicates {
    pub fn report(files: &[SourceFile]) -> Report {
        let mut predicates = EnumPredicates { enums: BTreeSet::new() };
        files.iter().for_each(|file| predicates.visit_file(file.syntax()));
        Report::new(
            "an enum method returning bool throws away the variant; return an enum or match in the enum's file",
            files.iter().flat_map(|file| predicates.violations(file)).collect(),
        )
    }

    fn violations(&self, file: &SourceFile) -> Vec<Violation> {
        file.syntax()
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Impl(block) if block.trait_.is_none() => Some(block),
                _ => None,
            })
            .filter(|block| self.is_enum(&block.self_ty))
            .flat_map(|block| Self::predicates(file.path(), block))
            .collect()
    }

    fn predicates(path: &str, block: &ItemImpl) -> Vec<Violation> {
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(function) => Some(&function.sig),
                _ => None,
            })
            .filter(|signature| match &signature.output {
                ReturnType::Type(_, returned) => Self::is_bool(returned),
                ReturnType::Default => false,
            })
            .map(|signature| {
                Violation::new(Site::spanned(path, signature.span()), BoolPredicate::new(&signature.ident))
            })
            .collect()
    }

    fn is_enum(&self, ty: &Type) -> bool {
        match ty {
            Type::Path(path) => {
                path.path.segments.last().is_some_and(|segment| self.enums.contains(&Name::from(&segment.ident)))
            }
            _ => false,
        }
    }

    fn is_bool(ty: &Type) -> bool {
        match ty {
            Type::Path(path) => path.path.is_ident("bool"),
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
    use crate::source_file::SourceFile;

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
    const FLAGGED: [&str; 1] = ["fn is_leg -> bool"];

    #[test]
    fn flags_inherent_enum_methods_returning_bool_only() {
        let file = SourceFile::parse("zoom.rs".to_owned(), SOURCE.to_owned()).expect("valid rust");
        let report = EnumPredicates::report(&[file]).to_string();
        let found: Vec<&str> = report.lines().skip(1).collect();
        assert_eq!(found.iter().map(|line| line.rsplit(": ").next().unwrap_or(line)).collect::<Vec<_>>(), FLAGGED);
    }
}
