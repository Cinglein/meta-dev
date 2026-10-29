use std::convert::Infallible;
use std::str::FromStr;

use crate::request::hook::hook_input::pushed_commits::patch::line::Line;

#[derive(Debug)]
pub struct Hunk {
    prefix_len: usize,
    first: Line,
    body: String,
}

impl Hunk {
    const SCRATCH: &'static str = concat!("/tmp/", "claude-");
    const MARK: char = '@';
    const ADDED: char = '+';
    const REMOVED: char = '-';
    const BODY: [char; 3] = [' ', '+', '-'];

    pub fn first_scratch_line(&self) -> Option<Line> {
        self.body
            .lines()
            .filter(|line| line.starts_with(Self::BODY))
            .scan(self.first, |next, line| {
                let at = *next;
                *next = if self.removes(line) { at } else { at.following() };
                Some(self.adds_scratch(line).map(|()| at))
            })
            .flatten()
            .next()
    }

    fn adds_scratch(&self, line: &str) -> Option<()> {
        let marks = line.get(..self.prefix_len)?;
        let text = line.get(self.prefix_len..)?;
        (marks.trim_start_matches(Self::ADDED).is_empty() && text.contains(Self::SCRATCH)).then_some(())
    }

    fn removes(&self, line: &str) -> bool {
        line.get(..self.prefix_len).is_some_and(|marks| marks.contains(Self::REMOVED))
    }
}

impl FromStr for Hunk {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (header, body) = text.split_once('\n').unwrap_or((text, ""));
        let extra = header.chars().take_while(|mark| *mark == Self::MARK).count();
        let added = header.split(' ').find_map(|word| word.strip_prefix(Self::ADDED));
        let first = added.and_then(|range| range.parse().ok()).unwrap_or_default();
        Ok(Self { prefix_len: extra.saturating_add(1), first, body: body.to_owned() })
    }
}
