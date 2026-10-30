#[derive(Clone, Copy)]
pub enum Closer {
    Paren,
    Tick,
}

impl Closer {
    pub fn symbol(self) -> char {
        match self {
            Self::Paren => ')',
            Self::Tick => '`',
        }
    }
}
