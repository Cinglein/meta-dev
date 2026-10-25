use rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};

use crate::line_number::LineNumber;
use crate::rule::report::Report;
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct NoComments;

impl NoComments {
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "comments are not allowed in this repository",
            files
                .iter()
                .flat_map(|file| {
                    Self::comment_lines(file.text())
                        .into_iter()
                        .map(move |line| Violation::new(Site::Line(file.path().to_owned(), line), "comment"))
                })
                .collect(),
        )
    }

    fn comment_lines(source: &str) -> Vec<LineNumber> {
        tokenize(source, FrontmatterAllowed::Yes)
            .scan(0, |offset, token| {
                let start = *offset;
                *offset += usize::try_from(token.len).ok()?;
                Some((start, token.kind))
            })
            .filter(|(_, kind)| matches!(kind, TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }))
            .map(|(start, _)| LineNumber::after_index(source[..start].matches('\n').count()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::NoComments;

    #[test]
    fn flags_line_block_and_doc_comments() {
        let source = "fn a() {}\n// one\nfn b() {}\n/* two */\n/// three\nfn c() {}\n";
        assert_eq!(
            NoComments::comment_lines(source).iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["2", "4", "5"]
        );
    }

    #[test]
    fn ignores_comment_syntax_inside_literals() {
        let source = "const URL: &str = \"https://example.com\";\nconst C: char = '/';\n";
        assert!(NoComments::comment_lines(source).is_empty());
    }
}
