use super::super::bracket::Bracket;

#[derive(Clone, Copy, Default)]
pub struct Depth(usize);

impl Depth {
    pub fn after(self, bracket: Option<Bracket>) -> Self {
        match bracket {
            Some(Bracket::Open) => Self(self.0.saturating_add(1)),
            Some(Bracket::Close) => Self(self.0.saturating_sub(1)),
            None => self,
        }
    }

    pub fn is_top(self) -> bool {
        self.0 == 0
    }
}
