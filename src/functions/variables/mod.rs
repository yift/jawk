mod at;
mod define;
mod get_variable;
mod set;

use crate::functions_definitions::FunctionsGroup;
use at::get as get_at;
use define::get as get_define;
use get_variable::get as get_get_variable;
use set::get as get_set;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("variables")
        .add_function(get_set())
        .add_function(get_get_variable())
        .add_function(get_define())
        .add_function(get_at())
}
