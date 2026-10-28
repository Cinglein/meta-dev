mod ignored;

use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use signal_hook::low_level::emulate_default_handler;

use super::group::Group;
use ignored::Ignored;

static WATCHED: Once = Once::new();
static RECEIVED: AtomicI32 = AtomicI32::new(0);
static ENDED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct Interrupts {
    received: &'static AtomicI32,
}

impl Interrupts {
    const WAIT: Duration = Duration::from_secs(2);
    const STEP: Duration = Duration::from_millis(5);

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
            let deadline = Instant::now() + Self::WAIT;
            let waits = std::iter::repeat_with(|| std::thread::sleep(Self::STEP));
            waits.take_while(|()| !ENDED.load(Ordering::SeqCst) && Instant::now() < deadline).for_each(drop);
            emulate_default_handler(signal)
        }));
    }

    fn forwarded(mut signals: Signals) -> JoinHandle<()> {
        std::thread::spawn(move || {
            signals.forever().for_each(|signal| {
                RECEIVED.store(signal, Ordering::SeqCst);
                Group::all_ended();
                ENDED.store(true, Ordering::SeqCst);
                drop(emulate_default_handler(signal));
            });
        })
    }
}
