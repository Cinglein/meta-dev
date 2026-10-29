use std::fmt;

use strum::VariantArray;
use tracing::field::{Field, Visit};

use crate::kind_tag::KindTag;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindCode(u16);

impl KindCode {
    pub const UNKNOWN: Self = Self(u16::MAX);

    pub fn new<Payload: VariantArray + PartialEq>(tag: KindTag, payload: &Payload) -> Self {
        Self(u16::from_le_bytes([Self::position(payload), Self::position(&tag)]))
    }

    pub fn word(self) -> u16 {
        self.0
    }

    pub fn tag(self) -> Option<KindTag> {
        let [_, tag] = self.0.to_le_bytes();
        KindTag::VARIANTS.get(usize::from(tag)).copied()
    }

    pub fn payload<Payload: VariantArray + Clone>(self) -> Option<Payload> {
        let [payload, _] = self.0.to_le_bytes();
        Payload::VARIANTS.get(usize::from(payload)).cloned()
    }

    fn position<Payload: VariantArray + PartialEq>(payload: &Payload) -> u8 {
        let index = Payload::VARIANTS.iter().position(|variant| variant == payload);
        index.and_then(|index| u8::try_from(index).ok()).unwrap_or(u8::MAX)
    }
}

impl From<u16> for KindCode {
    fn from(word: u16) -> Self {
        Self(word)
    }
}

impl Visit for KindCode {
    fn record_u64(&mut self, _: &Field, word: u64) {
        *self = u16::try_from(word).map_or(Self::UNKNOWN, Self);
    }

    fn record_debug(&mut self, _: &Field, _: &dyn fmt::Debug) {}
}
