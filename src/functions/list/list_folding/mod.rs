mod all;
mod any;
mod first;
mod join;
mod last;
mod sum;

use crate::functions_definitions::FunctionsGroup;
use all::get as get_all;
use any::get as get_any;
use first::get as get_first;
use join::get as get_join;
use last::get as get_last;
use sum::get as get_sum;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("list_folding")
        .add_function(get_sum())
        .add_function(get_any())
        .add_function(get_all())
        .add_function(get_join())
        .add_function(get_first())
        .add_function(get_last())
        .add_description_line("Function top fold a list into a single item")
}
