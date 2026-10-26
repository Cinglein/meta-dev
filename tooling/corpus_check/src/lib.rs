#[cfg(test)]
mod corpus;
mod lint_name;

pub use lint_name::LintName;

#[cfg(test)]
mod tests {
    use super::LintName;
    use super::corpus::Corpus;
    use settings::EmbeddedDefaults;
    use strum::IntoEnumIterator;
    use task::Policy;

    #[test]
    fn every_lint_reports_exactly_the_markers_of_every_corpus_case() {
        let corpus = Corpus::load().unwrap();
        let mismatch = corpus.mismatch(&Policy::defaults());
        assert!(mismatch.is_empty(), "{mismatch}");
        assert!(LintName::iter().all(|lint| corpus.expects(lint)));
    }
}
