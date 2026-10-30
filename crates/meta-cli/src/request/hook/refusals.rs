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
        self.input
            .tool()
            .err()
            .or_else(|| Self::modal(self.input))
            .or_else(|| Self::unleased(self.input, self.project))
            .or_else(|| self.project.and_then(|project| self.command(project)))
            .or_else(|| self.input.refusal())
            .or_else(|| self.project.and_then(|project| self.input.unpushable(project, Self::PUSH)))
    }

    fn modal(input: &HookInput) -> Option<Listed> {
        ScalePolicy::embedded_section(ScalePolicy::tools)
            .map_or_else(|why| Some(Listed::new(why)), |tools| input.modal(tools))
    }

    fn unleased(input: &HookInput, project: Option<&Project>) -> Option<Listed> {
        ScalePolicy::embedded_section(ScalePolicy::tools)
            .map_or_else(|why| Some(Listed::new(why)), |tools| input.agent_call(tools)?.refusal(project))
    }

    fn command(&self, project: &Project) -> Option<Listed> {
        let policy = ScalePolicy::read(project.root()).ok().unwrap_or_default();
        let lander = policy.queue().lander(self.input.announced_actor());
        self.input.command()?.refusal(project.config(), lander.as_ref())
    }
}
