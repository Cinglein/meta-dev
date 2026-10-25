mod short_lifetime;

use short_lifetime::ShortLifetime;
use syn::LifetimeParam;
use syn::visit::Visit;

use crate::rule::report::Report;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct NamedLifetimes;

impl NamedLifetimes {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "lifetimes are named with a word, never a letter",
            files
                .iter()
                .flat_map(|file| {
                    Self::short_lifetimes(file.syntax())
                        .into_iter()
                        .map(move |name| Violation::new(Site::spanned(file.path(), name.name().span()), name))
                })
                .collect(),
        )
    }

    fn short_lifetimes(file: &syn::File) -> Vec<ShortLifetime> {
        let mut visitor = ShortLifetimes::default();
        visitor.visit_file(file);
        visitor.0
    }
}

#[derive(Default)]
struct ShortLifetimes(Vec<ShortLifetime>);

impl<'ast> Visit<'ast> for ShortLifetimes {
    fn visit_lifetime_param(&mut self, parameter: &'ast LifetimeParam) {
        self.0.extend(ShortLifetime::try_from(&parameter.lifetime.ident));
    }
}

#[cfg(test)]
mod tests {
    use super::NamedLifetimes;

    const SOURCE: &str = "
struct Scan<'a, 'scan> { text: &'a str, path: &'scan str, owned: &'static str }
impl<'b> Scan<'b, '_> { fn view<'c>(&'c self) -> &'c str { self.text } }
";
    const FLAGGED: [&str; 3] = ["a", "b", "c"];

    #[test]
    fn flags_single_letter_lifetimes_where_they_are_declared() {
        let parsed = syn::parse_file(SOURCE).expect("valid rust");
        let names: Vec<String> =
            NamedLifetimes::short_lifetimes(&parsed).iter().map(|short| short.name().to_string()).collect();
        assert_eq!(names, FLAGGED);
    }
}
