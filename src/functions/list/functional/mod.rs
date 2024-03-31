mod filter;
mod flat_map;
mod fold;
mod group_by;
mod map;
mod sort_by;

use crate::functions_definitions::FunctionsGroup;
use filter::get as get_filter;
use flat_map::get as get_flat_map;
use fold::get as get_fold;
use group_by::get as get_group_by;
use map::get as get_map;
use sort_by::get as get_sort_by;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("list_functional")
        .add_function(get_filter())
        .add_function(get_group_by())
        .add_function(get_sort_by())
        .add_function(get_fold())
        .add_function(get_map())
        .add_function(get_flat_map())
        .add_description_line("Functional function over a list of items")
}
