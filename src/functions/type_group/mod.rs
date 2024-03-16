mod is_array;
mod is_bool;
mod is_empty;
mod is_null;
mod is_number;
mod is_object;
mod is_string;

use crate::functions_definitions::FunctionsGroup;
use is_array::get as get_is_array;
use is_bool::get as get_is_bool;
use is_empty::get as get_is_empty;
use is_null::get as get_is_null;
use is_number::get as get_is_number;
use is_object::get as get_is_object;
use is_string::get as get_is_string;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("type_group")
        .add_function(get_is_array())
        .add_function(get_is_object())
        .add_function(get_is_null())
        .add_function(get_is_bool())
        .add_function(get_is_number())
        .add_function(get_is_string())
        .add_function(get_is_empty())
}
