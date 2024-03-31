mod cast;
mod check_types;

use crate::functions_definitions::FunctionsGroup;
use cast::group as cast;
use check_types::group as check_types;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("types")
        .add_sub_group(check_types())
        .add_sub_group(cast())
}
