mod base63_decode;
mod concat;
mod env;
mod head;
mod parse_and_stringify;
mod regex;
mod split;
mod tail;

use crate::functions_definitions::FunctionsGroup;
use base63_decode::get as get_base63_decode;
use concat::get as get_concat;
use env::get as get_env;
use head::get as get_head;
use parse_and_stringify::group as parse;
use regex::group as get_regex;
use split::get as get_split;
use tail::get as get_tail;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("string")
        .add_function(get_env())
        .add_function(get_concat())
        .add_function(get_head())
        .add_function(get_tail())
        .add_function(get_split())
        .add_function(get_base63_decode())
        .add_sub_group(parse())
        .add_sub_group(get_regex())
}
