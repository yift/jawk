mod entries;
mod keys;
mod values;

use crate::functions_definitions::FunctionsGroup;
use entries::get as get_entries;
use keys::get as get_keys;
use values::get as get_values;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("object_to_list")
        .add_function(get_keys())
        .add_function(get_values())
        .add_function(get_entries())
        .add_description_line("Convert an object into a list")
}
