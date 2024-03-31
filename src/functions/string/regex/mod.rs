mod extract_regex_group;
mod match_regex;

use crate::functions_definitions::FunctionsGroup;
use extract_regex_group::get as get_extract_regex_group;
use match_regex::get as get_match_regex;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("regex")
        .add_function(get_match_regex())
        .add_function(get_extract_regex_group())
        .add_description_line("Regular expression functions")
}
