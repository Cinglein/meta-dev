mod clock_window;

use std::fmt;
use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use super::note::Note;
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

    pub fn poll(
        &self,
        refused: Option<impl fmt::Display>,
        wait: Duration,
        settings: &CiSettings,
    ) -> ControlFlow<Option<Note>> {
        std::thread::sleep(self.window().poll(refused, wait, settings)?);
        ControlFlow::Continue(())
    }
}
