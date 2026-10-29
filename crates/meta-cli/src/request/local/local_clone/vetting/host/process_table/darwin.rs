mod fd_vnode;
mod probe_failure;
mod process_dirs;
mod vnode_path;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use libproc::bsd_info::BSDInfo;
use libproc::file_info::{ListFDs, ProcFDType};
use libproc::proc_pid::{listpidinfo, pidinfo, pidpath};
use libproc::processes::{ProcFilter, pids_by_type};

use super::held::Held;
use super::scan::Scan;
use crate::request::local::local_clone::vetting::cause::Cause;
use fd_vnode::FdVnode;
use probe_failure::ProbeFailure;
use process_dirs::ProcessDirs;

#[derive(Debug)]
pub struct Darwin {
    pid: i32,
}

impl Darwin {
    const WHOLE: u64 = 0;

    pub fn held() -> Result<Held, Cause> {
        let own = i32::try_from(std::process::id()).map_err(Cause::new)?;
        let uid = pidinfo::<BSDInfo>(own, Self::WHOLE).map_err(Cause::new)?.pbi_uid;
        let owned = Self::listed(ProcFilter::ByUID { uid })?;
        Self::listed(ProcFilter::All)?.into_iter().map(|pid| Self { pid }.scanned(&owned)).collect()
    }

    fn listed(filter: ProcFilter) -> Result<Vec<i32>, Cause> {
        let pids = pids_by_type(filter).map_err(Cause::new)?;
        Ok(pids.into_iter().filter_map(|pid| i32::try_from(pid).ok()).collect())
    }

    fn scanned(&self, owned: &[i32]) -> Result<Scan, Cause> {
        self.opened().map(Scan::Opened).or_else(|failure| match failure.unexplained() {
            Some(cause) if owned.contains(&self.pid) && self.alive()? => Err(cause),
            Some(_) | None => Ok(Scan::Skipped),
        })
    }

    fn opened(&self) -> Result<Vec<PathBuf>, ProbeFailure> {
        let info = pidinfo::<BSDInfo>(self.pid, Self::WHOLE)?;
        let dirs = pidinfo::<ProcessDirs>(self.pid, Self::WHOLE)?;
        let table_len = usize::try_from(info.pbi_nfiles).map_err(Cause::new)?;
        let fds = listpidinfo::<ListFDs>(self.pid, table_len)?;
        let vnodes = fds.iter().filter(|fd| matches!(ProcFDType::from(fd.proc_fdtype), ProcFDType::VNode));
        let files = vnodes.map(|fd| FdVnode::read(self.pid, fd.proc_fd, table_len));
        let files = files.collect::<Result<Vec<_>, ProbeFailure>>()?.into_iter().flatten();
        let program = PathBuf::from(pidpath(self.pid)?);
        Ok(dirs.paths().chain(files).chain([program]).collect())
    }

    fn alive(&self) -> Result<bool, Cause> {
        Ok(Self::listed(ProcFilter::All)?.contains(&self.pid)
            && pidinfo::<BSDInfo>(self.pid, Self::WHOLE).map_or(true, |info| info.pbi_status != libc::SZOMB))
    }
}
