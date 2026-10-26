mod finding;

use finding::Finding;
use syn::visit::Visit;

use syn::{Ident, LifetimeParam};

use task::LineNumber;
use task::Name;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::{Floor, Policy};

#[derive(Debug)]
pub struct NamedLifetimes;

impl NamedLifetimes {
    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "lifetimes are named with a word, never a letter",
            files
                .iter()
                .flat_map(|file| {
                    Self::short_lifetimes(file.syntax(), policy)
                        .into_iter()
                        .map(move |name| {
                            Violation::new(
                                Site::Line(file.path().into(), LineNumber::from(name.span())),
                                Finding::ShortLifetime {
                                    name: Name::from(&name),
                                },
                            )
                        })
                })
                .collect(),
        )
    }

    fn short_lifetimes(file: &syn::File, policy: &Policy) -> Vec<Ident> {
        let mut visitor = ShortLifetimes {
            policy,
            found: Vec::new(),
        };
        visitor.visit_file(file);
        visitor.found
    }
}

struct ShortLifetimes<'scan> {
    policy: &'scan Policy,
    found: Vec<Ident>,
}

impl<'ast> Visit<'ast> for ShortLifetimes<'_> {
    fn visit_lifetime_param(&mut self, parameter: &'ast LifetimeParam) {
        let name = &parameter.lifetime.ident;
        self.found.extend(
            (!self
                .policy
                .floor(Floor::LifetimeName)
                .allows(Name::from(name).length()))
            .then(|| name.clone()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NamedLifetimes;
    use settings::EmbeddedDefaults;
    use task::{Policy, SourceFile};
    const SOURCE: &str = "
struct Scan<'a, 'scan> { text: &'a str, path: &'scan Path, owned: &'static str }
impl<'b> Scan<'b, '_> { fn view<'c>(&'c self) -> &'c str { self.text } }
";
    const REPORTED: [&str; 3] = [":2: 'a", ":3: 'b", ":3: 'c"];

    #[test]
    fn flags_single_letter_lifetimes_where_they_are_declared() {
        let file = SourceFile::parse("scan.rs", SOURCE.to_owned()).unwrap();
        let report = NamedLifetimes::report(core::slice::from_ref(&file), &Policy::defaults());
        assert!(report.lists_exactly(&REPORTED), "{report}");
    }
}
