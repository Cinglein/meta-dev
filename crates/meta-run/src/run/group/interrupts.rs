use std::sync::Once;
use std::thread::JoinHandle;

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use signal_hook::low_level::emulate_default_handler;

use super::Group;

static WATCHED: Once = Once::new();

#[derive(Debug)]
pub struct Interrupts;

impl Interrupts {
    pub fn watched() {
        WATCHED.call_once(|| drop(Signals::new([SIGINT, SIGTERM, SIGHUP]).map(Self::forwarded)));
    }

    fn forwarded(mut signals: Signals) -> JoinHandle<()> {
        std::thread::spawn(move || {
            signals.forever().for_each(|signal| {
                Group::all_ended();
                drop(emulate_default_handler(signal));
            });
        })
    }
}
