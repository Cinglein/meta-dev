use std::collections::BTreeSet;
use std::process::Command;
use std::time::{Duration, Instant};

use super::machine::Machine;
use super::pid_list::PidList;
use super::process_row::Pid;
use super::signal::Signal;
use super::target::Target;
use crate::request::local::local_settings::MonitorSettings;
use crate::request::timed_command::TimedCommand;

#[derive(Clone, Copy, Debug)]
pub struct LiveMachine<'settings> {
    settings: &'settings MonitorSettings,
}

impl<'settings> LiveMachine<'settings> {
    const PS: &'static str = "ps";
    const LISTING: [&'static str; 3] = ["x", "-o", "pid=,ppid=,pgid=,etime=,time=,%cpu=,command="];
    const LSOF: &'static str = "lsof";
    const CWD: [&'static str; 6] = ["-a", "-d", "cwd", "-F", "n", "-p"];
    const UPTIME: &'static str = "uptime";
    const KILL: &'static str = "kill";
    const END_OF_OPTIONS: &'static str = "--";
    const LOCALE: &'static str = "LC_ALL";
    const PLAIN: &'static str = "C";

    pub fn new(settings: &'settings MonitorSettings) -> Self {
        Self { settings }
    }

    fn timed(self, mut command: Command) -> TimedCommand {
        command.env(Self::LOCALE, Self::PLAIN);
        TimedCommand::new(command, self.settings.probe_deadline(Instant::now()))
    }
}

impl Machine for LiveMachine<'_> {
    fn table(&self) -> Option<String> {
        let mut listing = Command::new(Self::PS);
        listing.args(Self::LISTING);
        self.timed(listing).stdout().ok()
    }

    fn cwds(&self, pids: &PidList) -> Option<String> {
        let mut listing = Command::new(Self::LSOF);
        pids.appended(listing.args(Self::CWD));
        Some(self.timed(listing).finished().ok()?.output().to_owned())
    }

    fn uptime(&self) -> Option<String> {
        self.timed(Command::new(Self::UPTIME)).stdout().ok()
    }

    fn signal(&self, signal: Signal, targets: &BTreeSet<Target>) {
        let mut kill = Command::new(Self::KILL);
        kill.args([signal.flag(), Self::END_OF_OPTIONS]);
        let kill = targets.iter().fold(kill, |mut kill, target| {
            target.appended(&mut kill);
            kill
        });
        self.timed(kill).finished().ok();
    }

    fn pause(&self, grace: Duration) {
        std::thread::sleep(grace);
    }

    fn own_pid(&self) -> Pid {
        Pid::from(std::process::id())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
    use std::path::Path;
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::LiveMachine;
    use crate::request::local::local_settings::MonitorSettings;
    use crate::request::local::monitor::load_average::LoadAverage;
    use crate::request::local::monitor::machine::Machine;
    use crate::request::local::monitor::pid_list::PidList;
    use crate::request::local::monitor::process_row::{GroupId, Pid, ProcessRow};
    use crate::request::local::monitor::signal::Signal;
    use crate::request::local::monitor::target::Target;

    const SLEEP: &str = "sleep";
    const SECONDS: &str = "10";
    const OWN_GROUP: i32 = 0;
    const PLACES: [&str; 3] = ["/c", "/w", "/h"];
    const GRACE: Duration = Duration::from_millis(1);
    const KILLED: i32 = 9;

    #[test]
    fn the_live_machine_lists_a_child_with_its_cwd_and_kills_its_group() {
        let root = tempfile::tempdir().unwrap();
        let mut child =
            Command::new(SLEEP).arg(SECONDS).current_dir(root.path()).process_group(OWN_GROUP).spawn().unwrap();
        let [checkout, worktrees, home] = PLACES.map(Path::new);
        let settings = MonitorSettings::merged(toml::Table::new(), checkout, worktrees, home).unwrap();
        let (machine, pid) = (LiveMachine::new(&settings), Pid::from(child.id()));
        let table = machine.table().unwrap_or_default();
        let cwds = machine.cwds(&PidList::new(vec![pid])).unwrap_or_default();
        let load = machine.uptime().and_then(|uptime| uptime.parse::<LoadAverage>().ok());
        let started = Instant::now();
        machine.pause(GRACE);
        let paused = started.elapsed();
        machine.signal(Signal::Kill, &BTreeSet::from([Target::Group(GroupId::from(pid))]));
        let status = child.wait().unwrap();
        let parent =
            table.lines().filter_map(|line| line.parse::<ProcessRow>().ok()).find_map(|row| row.parent_of(pid));
        let place = std::fs::canonicalize(root.path()).unwrap();
        assert_eq!(parent, Some(machine.own_pid()));
        assert!(cwds.contains(&*place.to_string_lossy()) && load.is_some());
        assert!(status.signal() == Some(KILLED) && paused >= GRACE);
    }
}
