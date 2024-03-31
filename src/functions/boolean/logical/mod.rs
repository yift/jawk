mod and;
mod not;
mod or;
mod xor;

use crate::functions_definitions::FunctionsGroup;
use and::get as get_and;
use not::get as get_not;
use or::get as get_or;
use xor::get as get_xor;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("logical")
        .add_function(get_and())
        .add_function(get_or())
        .add_function(get_xor())
        .add_function(get_not())
        .add_description_line("Logical function")
}
