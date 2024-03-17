mod and;
mod eq;
mod gt;
mod gte;
mod lt;
mod lte;
mod neq;
mod not;
mod or;
mod xor;

use crate::functions_definitions::FunctionsGroup;
use and::get as get_and;
use eq::get as get_eq;
use gt::get as get_gt;
use gte::get as get_gte;
use lt::get as get_lt;
use lte::get as get_lte;
use neq::get as get_neq;
use not::get as get_not;
use or::get as get_or;
use xor::get as get_xor;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("boolean")
        .add_function(get_eq())
        .add_function(get_neq())
        .add_function(get_lt())
        .add_function(get_lte())
        .add_function(get_gte())
        .add_function(get_gt())
        .add_function(get_and())
        .add_function(get_or())
        .add_function(get_xor())
        .add_function(get_not())
}
