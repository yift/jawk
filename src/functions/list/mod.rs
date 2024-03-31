mod functional;
mod list_folding;
mod list_manipulations;
mod list_producers;

use crate::functions_definitions::FunctionsGroup;
use functional::group as functional;
use list_folding::group as list_folding;
use list_manipulations::group as list_manipluations;
use list_producers::group as list_producers;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("list")
        .add_sub_group(list_producers())
        .add_sub_group(list_manipluations())
        .add_sub_group(functional())
        .add_sub_group(list_folding())
}
