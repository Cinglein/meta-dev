use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum GateState {
    Success,
    Failure,
}
