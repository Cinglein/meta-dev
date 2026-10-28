use derive_more::{Display, From};

#[derive(Debug, Display, From)]
pub struct KeyName(&'static str);
