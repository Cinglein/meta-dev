use super::tool::Tool;
use crate::command_line::CargoArgument;

#[derive(Debug)]
pub struct CargoTool;

impl Tool for CargoTool {
    fn whole_workspace() -> Option<CargoArgument> {
        Some(CargoArgument::Workspace)
    }
}
