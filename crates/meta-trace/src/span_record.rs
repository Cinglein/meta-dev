use std::ops::Range;

use serde::ser::{Serialize, SerializeStruct as _, Serializer};

use crate::kind_code::KindCode;
use crate::lane::Lane;
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
    lane: Lane,
}

impl SpanRecord {
    pub const LEN: usize = 36;
    const EVENT: &'static str = "event";
    const FIELDS_LEN: usize = 6;
    const COMPLETE: &'static str = "X";

    #[must_use]
    pub fn new(id: SpanId, parent: Option<SpanId>, kind: SpanKind, start: Micros) -> Self {
        Self { id, parent, kind, start, end: start, lane: Lane::default() }
    }

    #[must_use]
    pub fn ended(self, end: Micros) -> Self {
        Self { end, ..self }
    }

    pub(crate) fn on_lane(self, lane: Lane) -> Self {
        Self { lane, ..self }
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
        let halves = [KindCode::from(span.kind).word(), span.lane.word()];
        words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .chain(halves.iter().flat_map(|half| half.to_le_bytes()))
            .collect()
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
        let half =
            |bytes| <[u8]>::split_first_chunk(bytes).map(|(half, rest)| (u16::from_le_bytes(*half), rest)).ok_or(());
        let (code, rest) = half(rest)?;
        let (lane, _) = half(rest)?;
        let kind = SpanKind::try_from(KindCode::from(code))?;
        let span = Self::new(SpanId::read(id).ok_or(())?, SpanId::read(parent), kind, Micros::new(start));
        Ok(span.ended(Micros::new(end)).on_lane(Lane::new(lane)))
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
        event.serialize_field("tid", &self.lane.word())?;
        event.end()
    }
}
