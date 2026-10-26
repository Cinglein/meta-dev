use crate::command_line::CargoArgument;

pub trait Tool {
    fn whole_workspace() -> Option<CargoArgument>;
}
