#[derive(Clone, Copy, Debug)]
pub struct Position(usize);

impl Position {
    pub fn between(segment: &[String], rest: &[String]) -> Self {
        Self(segment.len().saturating_sub(rest.len()))
    }

    pub fn index(self) -> usize {
        self.0
    }
}
