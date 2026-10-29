use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum AlertArgument {
    #[strum(serialize = "api")]
    Api,
    #[strum(serialize = "--paginate")]
    Paginate,
    #[strum(serialize = "--jq")]
    Jq,
}
