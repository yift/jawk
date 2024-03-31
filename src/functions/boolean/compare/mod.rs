mod eq;
mod gt;
mod gte;
mod lt;
mod lte;
mod neq;

use crate::functions_definitions::FunctionsGroup;
use eq::get as get_eq;
use gt::get as get_gt;
use gte::get as get_gte;
use lt::get as get_lt;
use lte::get as get_lte;
use neq::get as get_neq;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("compare")
        .add_function(get_eq())
        .add_function(get_neq())
        .add_function(get_lt())
        .add_function(get_lte())
        .add_function(get_gte())
        .add_function(get_gt())
        .add_description_line("Compatison functions")
}
