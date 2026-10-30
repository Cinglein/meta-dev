#[derive(Clone, Copy, Default)]
pub struct Skip(usize);

impl Skip {
    pub fn past(rest: &[char], tail: &[char]) -> Self {
        Self((rest.len() - tail.len()).saturating_sub(1))
    }

    pub fn pending(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }
}
