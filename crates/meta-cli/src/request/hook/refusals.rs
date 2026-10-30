use super::HookInput;
use super::hook_input::{ShellCommand, Target};
use crate::request::output::Listed;
use crate::request::project::{Config, Project};
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
        Self::local(self.input, self.project)
            .or_else(|| self.input.command().and_then(|command| self.targeted(&command)))
    }

    fn local(input: &HookInput, project: Option<&Project>) -> Option<Listed> {
        Self::modal(input)
            .or_else(|| project.and_then(|project| Self::guarded(input, project.config(), Some(project))))
            .or_else(|| input.refusal())
            .or_else(|| project.and_then(|project| input.unpushable(project, Self::PUSH)))
    }

    fn targeted(&self, command: &ShellCommand) -> Option<Listed> {
        let target = Target::resolved_from(command, &self.input.cwd(), self.project);
        let named = target.projects().iter().find_map(|named| Self::local(self.input, Some(named)));
        named.or_else(|| target.strictest().and_then(|config| Self::guarded(self.input, config, self.project)))
    }

    fn modal(input: &HookInput) -> Option<Listed> {
        ScalePolicy::embedded_section(ScalePolicy::tools)
            .map_or_else(|why| Some(Listed::new(why)), |tools| input.modal(tools))
    }

    fn guarded(input: &HookInput, config: &Config, project: Option<&Project>) -> Option<Listed> {
        let policy = project.and_then(|project| ScalePolicy::read(project.root()).ok()).unwrap_or_default();
        let lander = policy.queue().lander(input.announced_actor());
        input.command()?.refusal(config, lander.as_ref())
    }
}
