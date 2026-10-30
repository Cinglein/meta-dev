use super::HookInput;
use crate::request::output::Listed;
use crate::request::project::Project;
use crate::request::scale_policy::ScalePolicy;

pub struct Refusals<'call> {
    input: &'call HookInput,
    project: Option<&'call Project>,
}

impl<'call> Refusals<'call> {
    const PUSH: &'static str = "push";

    pub fn new(input: &'call HookInput, project: Option<&'call Project>) -> Self {
        Self { input, project }
    }

    pub fn first(&self) -> Option<Listed> {
        Self::modal(self.input)
            .or_else(|| self.command())
            .or_else(|| self.input.refusal())
            .or_else(|| self.project.and_then(|project| self.input.unpushable(project, Self::PUSH)))
    }

    fn modal(input: &HookInput) -> Option<Listed> {
        ScalePolicy::embedded_section(ScalePolicy::tools)
            .map_or_else(|why| Some(Listed::new(why)), |tools| input.modal(tools))
    }

    fn command(&self) -> Option<Listed> {
        let guarded = self.project.and_then(|project| self.guarded(project));
        guarded.or_else(|| self.input.shell_breach(self.project.map(Project::root)))
    }

    fn guarded(&self, project: &Project) -> Option<Listed> {
        let policy = ScalePolicy::read(project.root()).ok().unwrap_or_default();
        let lander = policy.queue().lander(self.input.announced_actor());
        self.input.command()?.refusal(project.config(), lander.as_ref())
    }
}
