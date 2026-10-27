use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanField {
    Label,
    Program,
    Args,
    Network,
    Exit,
}

impl SpanField {
    const ALL: [Self; 5] = [Self::Label, Self::Program, Self::Args, Self::Network, Self::Exit];

    pub fn name(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Program => "program",
            Self::Args => "args",
            Self::Network => "network",
            Self::Exit => "exit",
        }
    }
}

impl FromStr for SpanField {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, ()> {
        Self::ALL.into_iter().find(|field| field.name() == name).ok_or(())
    }
}
