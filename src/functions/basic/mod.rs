mod condition;
mod default;
mod get;
mod pipe;
mod size;
mod stringify;
mod sub;
mod take;
mod take_last;

use crate::functions_definitions::FunctionsGroup;
use condition::get as get_condition;
use default::get as get_default;
use get::get as get_get;
use pipe::get as get_pipe;
use size::get as get_size;
use stringify::get as get_stringify;
use sub::get as get_sub;
use take::get as get_take;
use take_last::get as get_take_last;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("basic")
        .add_function(get_get())
        .add_function(get_pipe())
        .add_function(get_size())
        .add_function(get_take())
        .add_function(get_take_last())
        .add_function(get_sub())
        .add_function(get_default())
        .add_function(get_condition())
        .add_function(get_stringify())
}
