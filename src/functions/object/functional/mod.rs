mod filter_keys;
mod filter_values;
mod map_keys;
mod map_values;

use crate::functions_definitions::FunctionsGroup;
use filter_keys::get as get_filter_keys;
use filter_values::get as get_filter_values;
use map_keys::get as get_map_keys;
use map_values::get as get_map_values;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("functional_object")
        .add_function(get_filter_keys())
        .add_function(get_filter_values())
        .add_function(get_map_values())
        .add_function(get_map_keys())
        .add_description_line("Functional function over an object")
}
