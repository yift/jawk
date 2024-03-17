mod add;
mod divide;
mod reminder;
mod take_away;
mod times;

use crate::functions_definitions::FunctionsGroup;
use add::get as get_add;
use divide::get as get_divide;
use reminder::get as get_reminder;
use take_away::get as get_take_away;
use times::get as get_times;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("number")
        .add_function(get_add())
        .add_function(get_take_away())
        .add_function(get_times())
        .add_function(get_divide())
        .add_function(get_reminder())
}
