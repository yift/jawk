mod entries;
mod filter_keys;
mod filter_values;
mod insert_if_absent;
mod keys;
mod map_keys;
mod map_values;
mod put;
mod replace_if_exists;
mod sort_by_keys;
mod sort_by_values;
mod sort_by_values_by;
mod values;

use crate::functions_definitions::FunctionsGroup;
use entries::get as get_entries;
use filter_keys::get as get_filter_keys;
use filter_values::get as get_filter_values;
use insert_if_absent::get as get_insert_if_absent;
use keys::get as get_keys;
use map_keys::get as get_map_keys;
use map_values::get as get_map_values;
use put::get as get_put;
use replace_if_exists::get as get_replace_if_exists;
use sort_by_keys::get as get_sort_by_keys;
use sort_by_values::get as get_sort_by_values;
use sort_by_values_by::get as get_sort_by_values_by;
use values::get as get_values;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("object")
        .add_function(get_keys())
        .add_function(get_values())
        .add_function(get_entries())
        .add_function(get_sort_by_keys())
        .add_function(get_sort_by_values())
        .add_function(get_sort_by_values_by())
        .add_function(get_filter_keys())
        .add_function(get_filter_values())
        .add_function(get_map_values())
        .add_function(get_map_keys())
        .add_function(get_put())
        .add_function(get_insert_if_absent())
        .add_function(get_replace_if_exists())
}
