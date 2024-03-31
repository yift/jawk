mod sort_by_keys;
mod sort_by_values;
mod sort_by_values_by;

use crate::functions_definitions::FunctionsGroup;
use sort_by_keys::get as get_sort_by_keys;
use sort_by_values::get as get_sort_by_values;
use sort_by_values_by::get as get_sort_by_values_by;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("sort_objects")
        .add_function(get_sort_by_keys())
        .add_function(get_sort_by_values())
        .add_function(get_sort_by_values_by())
        .add_description_line("Sort an object")
}
