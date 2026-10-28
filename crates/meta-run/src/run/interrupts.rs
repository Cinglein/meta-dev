mod ignored;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, Once, PoisonError};
use std::thread::JoinHandle;

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use signal_hook::low_level::emulate_default_handler;

use super::group::Group;
use ignored::Ignored;

static WATCHED: Once = Once::new();
static RECEIVED: AtomicI32 = AtomicI32::new(0);
static SWEEPING: Mutex<()> = Mutex::new(());

#[derive(Debug)]
pub struct Interrupts {
    received: &'static AtomicI32,
}

impl Interrupts {
    pub fn watched() -> Self {
        WATCHED.call_once(|| {
            let ignored = Ignored::at_start();
            let wanted = [SIGINT, SIGTERM, SIGHUP].into_iter().filter(|signal| ignored.spares(*signal).is_some());
            drop(Signals::new(wanted).map(Self::forwarded));
        });
        Self { received: &RECEIVED }
    }

    pub fn honored(self) {
        let signal = self.received.load(Ordering::SeqCst);
        drop((signal != 0).then(|| {
            let _swept = SWEEPING.lock().unwrap_or_else(PoisonError::into_inner);
            emulate_default_handler(signal)
        }));
    }

    fn forwarded(mut signals: Signals) -> JoinHandle<()> {
        std::thread::spawn(move || {
            signals.forever().for_each(|signal| {
                let sweeping = SWEEPING.lock().unwrap_or_else(PoisonError::into_inner);
                RECEIVED.store(signal, Ordering::SeqCst);
                Group::all_ended();
                drop(sweeping);
                drop(emulate_default_handler(signal));
            });
        })
    }
}
