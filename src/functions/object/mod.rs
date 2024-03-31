mod functional;
mod manipulate_object;
mod object_to_list;
mod sort_objects;

use crate::functions_definitions::FunctionsGroup;
use functional::group as functional;
use manipulate_object::group as manipulate_object;
use object_to_list::group as object_to_list;
use sort_objects::group as sort_objects;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("object")
        .add_sub_group(object_to_list())
        .add_sub_group(sort_objects())
        .add_sub_group(manipulate_object())
        .add_sub_group(functional())
}
