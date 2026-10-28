use super::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitWork {
    Routine,
    Claim,
}

impl GitWork {
    pub fn limit(self, routine: Millis, claim: Millis) -> Millis {
        match self {
            Self::Routine => routine,
            Self::Claim => claim,
        }
    }
}
