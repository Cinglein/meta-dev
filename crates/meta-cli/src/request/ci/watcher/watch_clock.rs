mod clock_window;

use std::time::{Duration, Instant};

use crate::request::project::CiSettings;
pub use clock_window::ClockWindow;

#[derive(Debug)]
pub struct WatchClock {
    started: Instant,
    deadline: Instant,
}

impl WatchClock {
    pub fn start(settings: &CiSettings) -> Self {
        let started = Instant::now();
        Self { started, deadline: started + settings.remaining(Duration::ZERO).unwrap_or_default() }
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn window(&self) -> ClockWindow {
        ClockWindow::new(self.elapsed(), self.deadline.checked_duration_since(Instant::now()))
    }

    pub fn pause(&self, wait: Duration, settings: &CiSettings) -> Option<()> {
        self.window().pause(wait, settings).map(std::thread::sleep)
    }
}
