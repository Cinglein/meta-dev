use strum::VariantArray;

use crate::kind_code::KindCode;
use crate::span_kind::SpanKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, VariantArray)]
pub enum KindTag {
    Command,
    Spawn,
    Network,
    Fs,
}

impl KindTag {
    pub fn kind(self, code: KindCode) -> Option<SpanKind> {
        match self {
            Self::Command => code.payload().map(SpanKind::Command),
            Self::Spawn => code.payload().map(SpanKind::Spawn),
            Self::Network => code.payload().map(SpanKind::Network),
            Self::Fs => Some(SpanKind::Fs),
        }
    }
}
