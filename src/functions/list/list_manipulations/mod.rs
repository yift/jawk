mod indexed;
mod pop;
mod pop_first;
mod push;
mod push_front;
mod reverse;
mod sort;
mod sort_unique;

use crate::functions_definitions::FunctionsGroup;
use indexed::get as get_indexed;
use pop::get as get_pop;
use pop_first::get as get_pop_first;
use push::get as get_push;
use push_front::get as get_push_front;
use reverse::get as get_reverse;
use sort::get as get_sort;
use sort_unique::get as get_sort_unique;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("list_manipluations")
        .add_function(get_sort())
        .add_function(get_sort_unique())
        .add_function(get_indexed())
        .add_function(get_push())
        .add_function(get_push_front())
        .add_function(get_reverse())
        .add_function(get_pop())
        .add_function(get_pop_first())
        .add_description_line("Function to manipulatre a list and create a new one")
}
