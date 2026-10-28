use rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};

use task::LineNumber;
use task::Report;
use task::Site;
use task::SourceFile;
use task::Violation;

#[derive(Debug)]
pub struct NoComments;

impl NoComments {
    const OUTER_ATTRIBUTE_LENGTH: usize = 3;
    const INNER_ATTRIBUTE_LENGTH: usize = 4;

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
        let tokens: Vec<(usize, usize, TokenKind)> = tokenize(source, FrontmatterAllowed::Yes)
            .scan(0, |offset, token| {
                let start = *offset;
                *offset += usize::try_from(token.len).ok()?;
                Some((start, *offset, token.kind))
            })
            .filter(|(_, _, kind)| kind != &TokenKind::Whitespace)
            .collect();
        let is_doc = |token: &(usize, usize, TokenKind)| &source[token.0..token.1] == "doc";
        let outer =
            tokens
                .windows(Self::OUTER_ATTRIBUTE_LENGTH)
                .filter_map(|window| match window {
                    [
                        (start, _, TokenKind::Pound),
                        (_, _, TokenKind::OpenBracket),
                        name,
                    ] if is_doc(name) => Some(*start),
                    _ => None,
                });
        let inner =
            tokens
                .windows(Self::INNER_ATTRIBUTE_LENGTH)
                .filter_map(|window| match window {
                    [
                        (start, _, TokenKind::Pound),
                        (_, _, TokenKind::Bang),
                        (_, _, TokenKind::OpenBracket),
                        name,
                    ] if is_doc(name) => Some(*start),
                    _ => None,
                });
        let comments = tokens.iter().filter_map(|(start, _, kind)| {
            matches!(
                kind,
                TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
            )
            .then_some(*start)
        });
        comments
            .chain(outer)
            .chain(inner)
            .map(|start| LineNumber::new(source[..start].matches('\n').count() + 1))
            .collect()
    }
}
