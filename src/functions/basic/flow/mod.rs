mod condition;
mod default;
mod pipe;

use crate::functions_definitions::FunctionsGroup;
use condition::get as get_condition;
use default::get as get_default;
use pipe::get as get_pipe;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("flow")
        .add_function(get_pipe())
        .add_function(get_default())
        .add_function(get_condition())
        .add_description_line("Contrlo flow functions")
}
