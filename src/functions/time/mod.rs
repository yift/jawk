mod format_time;
mod now;
mod parse_time;
mod parse_time_with_zone;

use crate::functions_definitions::FunctionsGroup;
use format_time::get as get_format_time;
use now::get as get_now;
use parse_time::get as get_parse_time;
use parse_time_with_zone::get as get_parse_time_with_zone;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("time")
        .add_function(get_now())
        .add_function(get_format_time())
        .add_function(get_parse_time_with_zone())
        .add_function(get_parse_time())
}
