mod get;
mod size;
mod sub;
mod take;
mod take_last;

use crate::functions_definitions::FunctionsGroup;
use get::get as get_get;
use size::get as get_size;
use sub::get as get_sub;
use take::get as get_take;
use take_last::get as get_take_last;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("collection")
        .add_function(get_get())
        .add_function(get_size())
        .add_function(get_take())
        .add_function(get_take_last())
        .add_function(get_sub())
        .add_description_line("Functions that allow to use list or maps as collections")
}
