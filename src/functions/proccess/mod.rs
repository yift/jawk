mod exec;
mod trigger;

use crate::functions_definitions::FunctionsGroup;
use exec::get as get_exec;
use trigger::get as get_trigger;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("proccess")
        .add_function(get_exec())
        .add_function(get_trigger())
}
