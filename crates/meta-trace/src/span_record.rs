use std::ops::Range;

use serde::ser::{Serialize, SerializeStruct as _, Serializer};

use crate::kind_code::KindCode;
use crate::micros::Micros;
use crate::span_id::SpanId;
use crate::span_kind::SpanKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanRecord {
    id: SpanId,
    parent: Option<SpanId>,
    kind: SpanKind,
    start: Micros,
    end: Micros,
}

impl SpanRecord {
    pub const LEN: usize = 34;
    const EVENT: &'static str = "event";
    const FIELDS_LEN: usize = 6;
    const COMPLETE: &'static str = "X";

    #[must_use]
    pub fn new(id: SpanId, parent: Option<SpanId>, kind: SpanKind, start: Micros) -> Self {
        Self { id, parent, kind, start, end: start }
    }

    #[must_use]
    pub fn ended(self, end: Micros) -> Self {
        Self { end, ..self }
    }

    #[must_use]
    pub fn span_id(&self) -> SpanId {
        self.id
    }

    #[must_use]
    pub fn parent(&self) -> Option<SpanId> {
        self.parent
    }

    #[must_use]
    pub fn kind(&self) -> SpanKind {
        self.kind
    }

    #[must_use]
    pub fn window(&self) -> Range<Micros> {
        self.start..self.end
    }

    #[must_use]
    pub fn duration(&self) -> Micros {
        self.end - self.start
    }
}

impl From<&SpanRecord> for Vec<u8> {
    fn from(span: &SpanRecord) -> Self {
        let words = [span.id.word(), span.parent.map_or(0, SpanId::word), span.start.micros(), span.end.micros()];
        let code = KindCode::from(span.kind).word().to_le_bytes();
        words.iter().flat_map(|word| word.to_le_bytes()).chain(code).collect()
    }
}

impl TryFrom<&[u8; SpanRecord::LEN]> for SpanRecord {
    type Error = ();

    fn try_from(bytes: &[u8; Self::LEN]) -> Result<Self, ()> {
        let all: &[u8] = bytes;
        let word =
            |bytes| <[u8]>::split_first_chunk(bytes).map(|(word, rest)| (u64::from_le_bytes(*word), rest)).ok_or(());
        let (id, rest) = word(all)?;
        let (parent, rest) = word(rest)?;
        let (start, rest) = word(rest)?;
        let (end, rest) = word(rest)?;
        let code = rest.first_chunk().map(|code| KindCode::from(u16::from_le_bytes(*code))).ok_or(())?;
        let span =
            Self::new(SpanId::read(id).ok_or(())?, SpanId::read(parent), SpanKind::try_from(code)?, Micros::new(start));
        Ok(span.ended(Micros::new(end)))
    }
}

impl Serialize for SpanRecord {
    fn serialize<Shown: Serializer>(&self, serializer: Shown) -> Result<Shown::Ok, Shown::Error> {
        let mut event = serializer.serialize_struct(Self::EVENT, Self::FIELDS_LEN)?;
        event.serialize_field("name", &self.kind.to_string())?;
        event.serialize_field("ph", Self::COMPLETE)?;
        event.serialize_field("ts", &self.start.micros())?;
        event.serialize_field("dur", &self.duration().micros())?;
        event.serialize_field("pid", &self.id.pid())?;
        event.serialize_field("tid", &self.id.pid())?;
        event.end()
    }
}
