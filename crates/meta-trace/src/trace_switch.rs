mod recorder;
mod unknown_switch;

use std::str::FromStr;

use crate::command_word::CommandWord;
use crate::span_kind::SpanKind;
use crate::trace::Trace;
use recorder::Recorder;
use unknown_switch::UnknownSwitch;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TraceSwitch {
    On,
    #[default]
    Off,
}

impl TraceSwitch {
    pub const VARIABLE: &'static str = "META_TRACE";
    const ON: &'static str = "on";
    const OFF: &'static str = "off";

    pub fn ambient() -> Result<Self, UnknownSwitch> {
        std::env::var(Self::VARIABLE).map_or(Ok(Self::Off), |value| value.parse())
    }

    pub fn recorded<Answer>(
        self,
        word: &CommandWord,
        answer: impl FnOnce() -> Answer,
        keep: impl FnOnce(Trace),
    ) -> Answer {
        let commanded = || {
            let _command = SpanKind::Command.entered(word);
            answer()
        };
        match self {
            Self::On => Recorder::default().recorded(commanded, keep),
            Self::Off => commanded(),
        }
    }
}

impl FromStr for TraceSwitch {
    type Err = UnknownSwitch;

    fn from_str(value: &str) -> Result<Self, UnknownSwitch> {
        match value {
            Self::ON => Ok(Self::On),
            "" | Self::OFF => Ok(Self::Off),
            _ => Err(UnknownSwitch),
        }
    }
}
