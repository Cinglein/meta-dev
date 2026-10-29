#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Enforcing,
    #[default]
    Reporting,
}
