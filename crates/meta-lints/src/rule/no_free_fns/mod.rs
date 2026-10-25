mod free_fn;

use free_fn::FreeFn;
use syn::{Ident, Item, ItemFn, ItemMod};

use crate::rule::report::Report;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct NoFreeFns;

impl NoFreeFns {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "free functions are not allowed; make them associated functions or methods",
            files
                .iter()
                .flat_map(|file| {
                    Self::free_fns(&file.syntax().items)
                        .into_iter()
                        .map(move |name| Violation::new(Site::spanned(file.path(), name.span()), FreeFn::new(&name)))
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
        module.content.as_ref().map_or_else(Vec::new, |(_, items)| Self::free_fns(items))
    }

    fn is_exempt(function: &ItemFn) -> bool {
        function.sig.ident == "main" || function.attrs.iter().any(|attribute| attribute.path().is_ident("test"))
    }
}

#[cfg(test)]
mod tests {
    use super::NoFreeFns;

    const SOURCE: &str = "
fn main() {}
fn helper() {}
struct S;
impl S { fn method(&self) {} fn associated() {} }
trait T { fn required(); fn provided() {} }
mod inner {
    fn nested() {}
    #[cfg(test)]
    mod tests {
        #[test]
        fn a_test() {}
        fn test_helper() {}
    }
}
";
    const FLAGGED: [&str; 3] = ["helper", "nested", "test_helper"];

    #[test]
    fn flags_module_level_functions_but_not_methods_main_or_tests() {
        let parsed = syn::parse_file(SOURCE).expect("valid rust");
        let names: Vec<String> = NoFreeFns::free_fns(&parsed.items).iter().map(ToString::to_string).collect();
        assert_eq!(names, FLAGGED);
    }
}
