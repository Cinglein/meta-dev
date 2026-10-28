use std::ffi::{OsStr, OsString};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::Reported;
use crate::run::Run;
use crate::tool::Tool;

#[derive(Clone, Debug)]
pub struct Port<Argv> {
    program: OsString,
    dir: PathBuf,
    argv: PhantomData<Argv>,
}

impl<Argv: Tool> Port<Argv> {
    #[must_use]
    pub fn new(dir: &Path) -> Self {
        Self::named(Argv::PROGRAM, dir)
    }

    #[must_use]
    pub fn named(program: impl AsRef<OsStr>, dir: &Path) -> Self {
        Self { program: program.as_ref().to_os_string(), dir: dir.to_path_buf(), argv: PhantomData }
    }

    #[must_use]
    pub fn run<Word: AsRef<OsStr>>(&self, call: Argv, operands: impl IntoIterator<Item = Word>) -> Run {
        let [prompt, off] = Run::NO_PROMPT;
        let run = Run::new(&self.program, Argv::POLICY).var(prompt, off).dir(&self.dir);
        run.args(call.words()).args(operands)
    }

    pub fn reported<Word: AsRef<OsStr>>(
        &self,
        call: Argv,
        operands: impl IntoIterator<Item = Word>,
        deadline: Instant,
    ) -> Reported {
        self.run(call, operands).reported(deadline)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::{Duration, Instant};

    use super::Port;
    use crate::command_line::CommandLine;
    use crate::environment::Environment;
    use crate::git_argv::GitArgv;
    use crate::meta_argv::MetaArgv;

    const MISSING: &str = "/nonexistent/git";
    const NO_PROMPT: &str = "\"GIT_TERMINAL_PROMPT\": Some(\"0\")";
    const UNREAD: [&str; 2] = ["`/nonexistent/git var` ran out of time", "/nonexistent/git var: entity not found"];
    const NO_OPERANDS: [&str; 0] = [];
    const META_LIST: [&str; 2] = ["meta", "list"];
    const PATIENT: Duration = Duration::from_secs(5);

    #[test]
    fn a_port_runs_its_argv_without_prompts_and_reports_a_late_or_unspawned_call_as_unread() {
        let port = Port::named(MISSING, Path::new(MISSING));
        let shown = format!("{:?}", Environment::from(&port.run(GitArgv::Var, NO_OPERANDS)));
        let [late, unspawned] = [Instant::now(), Instant::now() + PATIENT]
            .map(|deadline| port.reported(GitArgv::Var, NO_OPERANDS, deadline).map_err(|unread| unread.to_string()));
        let [late_shown, unspawned_shown] = UNREAD;
        assert!(shown.contains(NO_PROMPT) && late.is_err_and(|late| late == late_shown));
        let meta = CommandLine::from(&Port::new(Path::new(MISSING)).run(MetaArgv::List, NO_OPERANDS));
        assert!(unspawned.is_err_and(|unspawned| unspawned.starts_with(unspawned_shown)) && meta.words() == META_LIST);
    }
}
