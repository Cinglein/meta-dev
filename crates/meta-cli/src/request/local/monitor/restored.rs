use super::fault::Fault;
use super::monitor_state::MonitorState;

#[derive(Debug)]
pub struct Restored {
    state: MonitorState,
    notice: Option<Fault>,
}

impl Restored {
    pub fn intact(state: MonitorState) -> Self {
        Self { state, notice: None }
    }

    pub fn recovered(notice: Fault) -> Self {
        Self { state: MonitorState::default(), notice: Some(notice) }
    }

    pub fn state(&self) -> &MonitorState {
        &self.state
    }

    pub fn notice(&self) -> Option<&Fault> {
        self.notice.as_ref()
    }
}
