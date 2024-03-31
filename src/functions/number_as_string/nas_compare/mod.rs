mod eq;
mod gt;
mod gte;
mod lt;
mod lte;
mod neq;
mod sort_by;

use crate::functions_definitions::FunctionsGroup;
use eq::get as get_eq;
use gt::get as get_gt;
use gte::get as get_gte;
use lt::get as get_lt;
use lte::get as get_lte;
use neq::get as get_neq;
use sort_by::get as get_sort_by;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("nas_compare")
        .add_description_line("Number As String comparison functions.")
        .add_function(get_eq())
        .add_function(get_neq())
        .add_function(get_gt())
        .add_function(get_lt())
        .add_function(get_gte())
        .add_function(get_lte())
        .add_function(get_sort_by())
}
