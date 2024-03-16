mod all;
mod any;
mod cross;
mod filter;
mod first;
mod flat_map;
mod fold;
mod group_by;
mod indexed;
mod join;
mod last;
mod map;
mod pop;
mod pop_first;
mod push;
mod push_front;
mod range;
mod reverese;
mod sort;
mod sort_by;
mod sort_unique;
mod sum;
mod zip;

use crate::functions_definitions::FunctionsGroup;
use all::get as get_all;
use any::get as get_any;
use cross::get as get_cross;
use filter::get as get_filter;
use first::get as get_first;
use flat_map::get as get_flat_map;
use fold::get as get_fold;
use group_by::get as get_group_by;
use indexed::get as get_indexed;
use join::get as get_join;
use last::get as get_last;
use map::get as get_map;
use pop::get as get_pop;
use pop_first::get as get_pop_first;
use push::get as get_push;
use push_front::get as get_push_front;
use range::get as get_range;
use reverese::get as get_reverese;
use sort::get as get_sort;
use sort_by::get as get_sort_by;
use sort_unique::get as get_sort_unique;
use sum::get as get_sum;
use zip::get as get_zip;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("list")
        .add_function(get_filter())
        .add_function(get_sort())
        .add_function(get_sort_unique())
        .add_function(get_group_by())
        .add_function(get_sort_by())
        .add_function(get_sum())
        .add_function(get_fold())
        .add_function(get_any())
        .add_function(get_all())
        .add_function(get_join())
        .add_function(get_first())
        .add_function(get_last())
        .add_function(get_map())
        .add_function(get_indexed())
        .add_function(get_flat_map())
        .add_function(get_range())
        .add_function(get_zip())
        .add_function(get_cross())
        .add_function(get_push())
        .add_function(get_push_front())
        .add_function(get_reverese())
        .add_function(get_pop())
        .add_function(get_pop_first())
}
