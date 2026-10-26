use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum GhArgument {
    #[strum(serialize = "api")]
    Api,
    #[strum(serialize = "graphql")]
    Graphql,
    #[strum(serialize = "-f")]
    Field,
    #[strum(serialize = "-F")]
    TypedField,
    #[strum(serialize = "--jq")]
    Jq,
}
