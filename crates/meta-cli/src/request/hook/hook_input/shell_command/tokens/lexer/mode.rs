#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Plain,
    Single,
    Double,
    Comment,
    ArithmeticPlain,
    ArithmeticDouble,
}
