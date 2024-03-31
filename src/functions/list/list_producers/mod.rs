mod cross;
mod range;
mod zip;

use crate::functions_definitions::FunctionsGroup;
use cross::get as get_cross;
use range::get as get_range;
use zip::get as get_zip;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("list_producers")
        .add_function(get_range())
        .add_function(get_zip())
        .add_function(get_cross())
        .add_description_line("Functions to create lists")
}
