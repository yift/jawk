mod insert_if_absent;
mod put;
mod replace_if_exists;

use crate::functions_definitions::FunctionsGroup;
use insert_if_absent::get as get_insert_if_absent;
use put::get as get_put;
use replace_if_exists::get as get_replace_if_exists;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("manipulate_object")
        .add_function(get_put())
        .add_function(get_insert_if_absent())
        .add_function(get_replace_if_exists())
        .add_description_line("Manipulate objects to pruduce a new object")
}
