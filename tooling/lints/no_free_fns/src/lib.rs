mod finding;

use finding::Finding;
use syn::{Ident, Item, ItemFn, ItemMod};

use task::LineNumber;
use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct NoFreeFns;

impl NoFreeFns {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "free functions are not allowed; make them associated functions or methods",
            files
                .iter()
                .flat_map(|file| {
                    Self::free_fns(&file.syntax().items)
                        .into_iter()
                        .map(move |name| {
                            Violation::new(
                                Site::Line(file.path().into(), LineNumber::from(name.span())),
                                Finding::FreeFn {
                                    name: Name::from(&name),
                                },
                            )
                        })
                })
                .collect(),
        )
    }

    fn free_fns(items: &[Item]) -> Vec<Ident> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Fn(function) if !Self::is_exempt(function) => {
                    vec![function.sig.ident.clone()]
                }
                Item::Mod(module) => Self::nested(module),
                _ => Vec::new(),
            })
            .collect()
    }

    fn nested(module: &ItemMod) -> Vec<Ident> {
        module
            .content
            .as_ref()
            .map_or_else(Vec::new, |(_, items)| Self::free_fns(items))
    }

    fn is_exempt(function: &ItemFn) -> bool {
        function.sig.ident == "main"
            || function
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("test"))
    }
}

#[cfg(test)]
mod tests {
    use super::NoFreeFns;
    use task::SourceFile;
    const SOURCE: &str = "
fn main() {} fn helper() {}
struct S; impl S { fn method(&self) {} fn associated() {} }
trait T { fn required(); fn provided() {} }
mod inner { fn nested() {} #[cfg(test)] mod tests { #[test] fn a_test() {} fn test_helper() {} } }
";
    const REPORTED: [&str; 3] = [":2: fn helper", ":5: fn nested", ":5: fn test_helper"];

    #[test]
    fn flags_module_level_functions_but_not_methods_main_or_tests() {
        let file = SourceFile::parse("free.rs", SOURCE.to_owned()).unwrap();
        let report = NoFreeFns::report(core::slice::from_ref(&file));
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
