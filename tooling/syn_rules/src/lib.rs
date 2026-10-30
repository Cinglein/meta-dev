mod field_access;
mod macro_expressions;

pub use field_access::FieldAccess;
pub use macro_expressions::MacroExpressions;

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::visit::Visit;
    use syn::{Block, Expr, Macro};

    use super::{FieldAccess, MacroExpressions};

    const BODIES: [(&str, &str); 7] = [
        ("{ self.a }", "Projected a true"),
        ("{ &self.b }", "Lent b false"),
        ("{ self.c = value; }", "Assigned c false"),
        ("{ other.a }", "none"),
        ("{ &other.b }", "none"),
        ("{ other.c = value; }", "none"),
        ("{ self.a; self.b }", "none"),
    ];
    const MACROS: [(&str, usize); 3] = [("format!(\"{}\", self.a, b)", 3), ("vec![a; 2]", 0), ("m!()", 0)];
    const NONE: &str = "none";

    #[derive(Default)]
    struct Visits(usize);

    impl Visits {
        fn described(body: &str) -> String {
            let block: Block = syn::parse_str(body).unwrap();
            FieldAccess::classify(&block).map_or_else(
                || NONE.to_owned(),
                |access| {
                    let kind = format!("{access:?}");
                    format!(
                        "{} {} {}",
                        kind.split('(').next().unwrap_or_default(),
                        access.member().to_token_stream(),
                        access.projection().is_some()
                    )
                },
            )
        }

        fn visited(text: &str) -> (usize, usize) {
            let invocation: Macro = syn::parse_str(text).unwrap();
            let (mut parsed, mut kept) = (Visits::default(), Visits::default());
            MacroExpressions::visit_parsed(&invocation, &mut parsed);
            if let Some(arguments) = MacroExpressions::parsed(&invocation) {
                arguments.visit_with(&mut kept);
            }
            (parsed.0, kept.0)
        }
    }

    impl<'ast> Visit<'ast> for Visits {
        fn visit_expr(&mut self, _: &'ast Expr) {
            self.0 += 1;
        }
    }

    #[test]
    fn only_a_lone_self_field_read_borrow_or_assignment_is_a_field_access() {
        let wrong: Vec<(&str, String)> = BODIES
            .iter()
            .map(|(body, expected)| (*body, Visits::described(body), *expected))
            .filter(|(_, found, expected)| found != expected)
            .map(|(body, found, _)| (body, found))
            .collect();
        assert!(wrong.is_empty(), "{wrong:?}");
    }

    #[test]
    fn macro_arguments_that_parse_as_expressions_are_visited_and_others_are_skipped() {
        let wrong: Vec<(&str, (usize, usize))> = MACROS
            .iter()
            .map(|(text, expected)| (*text, Visits::visited(text), *expected))
            .filter(|(_, found, expected)| *found != (*expected, *expected))
            .map(|(text, found, _)| (text, found))
            .collect();
        assert!(wrong.is_empty(), "{wrong:?}");
    }
}
