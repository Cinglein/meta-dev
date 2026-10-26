use rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};

use task::LineNumber;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct NoComments;

impl NoComments {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "comments are not allowed in this repository",
            files
                .iter()
                .flat_map(|file| {
                    Self::comment_lines(file.text())
                        .into_iter()
                        .map(move |line| {
                            Violation::new(Site::Line(file.path().into(), line), "comment")
                        })
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
            .filter(|(_, kind)| {
                matches!(
                    kind,
                    TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
                )
            })
            .map(|(start, _)| LineNumber::new(source[..start].matches('\n').count() + 1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{LineNumber, NoComments};
    #[test]
    fn flags_line_block_and_doc_comments() {
        let source = "fn a() {}\n// one\nfn b() {}\n/* two */\n/// three\nfn c() {}\n";
        assert_eq!(
            NoComments::comment_lines(source),
            [2, 4, 5].map(LineNumber::new)
        );
    }

    #[test]
    fn ignores_comment_syntax_inside_literals() {
        let source = "const URL: &str = \"https://example.com\";\nconst C: char = '/';\n";
        assert_eq!(NoComments::comment_lines(source), []);
    }
}
