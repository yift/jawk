mod base63_decode;
mod concat;
mod env;
mod extract_regex_group;
mod head;
mod match_regex;
mod parse;
mod parse_selection;
mod split;
mod tail;

use crate::functions_definitions::FunctionsGroup;
use base63_decode::get as get_base63_decode;
use concat::get as get_concat;
use env::get as get_env;
use extract_regex_group::get as get_extract_regex_group;
use head::get as get_head;
use match_regex::get as get_match_regex;
use parse::get as get_parse;
use parse_selection::get as get_parse_selection;
use split::get as get_split;
use tail::get as get_tail;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("string")
        .add_function(get_parse())
        .add_function(get_parse_selection())
        .add_function(get_env())
        .add_function(get_concat())
        .add_function(get_head())
        .add_function(get_tail())
        .add_function(get_split())
        .add_function(get_match_regex())
        .add_function(get_extract_regex_group())
        .add_function(get_base63_decode())
}
