use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum GhKey {
    Query,
    Owner,
    Name,
    Number,
}
