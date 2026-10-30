use strum::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    Tasks,
    Issues,
    Concerns,
    Projects,
}
