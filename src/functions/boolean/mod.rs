mod compare;
mod logical;

use crate::functions_definitions::FunctionsGroup;
use compare::group as compare;
use logical::group as logical;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("boolean")
        .add_sub_group(compare())
        .add_sub_group(logical())
}
