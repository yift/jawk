mod as_array;
mod as_bool;
mod as_number;
mod as_object;
mod as_string;

use crate::functions_definitions::FunctionsGroup;
use as_array::get as as_array;
use as_bool::get as as_bool;
use as_number::get as as_number;
use as_object::get as as_object;
use as_string::get as as_string;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("cast")
        .add_function(as_bool())
        .add_function(as_number())
        .add_function(as_string())
        .add_function(as_array())
        .add_function(as_object())
        .add_description_line("Cast object to a predefine type")
}
