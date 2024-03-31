mod parse;
mod parse_selection;
mod stringify;

use crate::functions_definitions::FunctionsGroup;
use parse::get as get_parse;
use parse_selection::get as get_parse_selection;
use stringify::get as get_stringify;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("parse_and_stringify")
        .add_function(get_stringify())
        .add_function(get_parse())
        .add_function(get_parse_selection())
        .add_description_line(
            "Function to parse from a string or to convert an object into a string",
        )
}
