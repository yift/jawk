mod collection;
mod flow;

use crate::functions_definitions::FunctionsGroup;
use collection::group as collection;
use flow::group as flow;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("basic")
        .add_sub_group(flow())
        .add_sub_group(collection())
}
