mod abs;
mod add;
mod divide;
mod eq;
mod gt;
mod gte;
mod lt;
mod lte;
mod neq;
mod normelize;
mod reminder;
mod round;
mod sort_by;
mod take_away;
mod times;
mod to_big_decimal;

use crate::functions_definitions::FunctionsGroup;
use abs::get as get_abs;
use add::get as get_add;
use divide::get as get_divide;
use eq::get as get_eq;
use gt::get as get_gt;
use gte::get as get_gte;
use lt::get as get_lt;
use lte::get as get_lte;
use neq::get as get_neq;
use normelize::get as get_normelize;
use reminder::get as get_reminder;
use round::get as get_round;
use sort_by::get as get_sort_by;
use take_away::get as get_take_away;
use times::get as get_times;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("nas")
        .add_description_line("NAS - Number As String, i.e. number that represent as string, like \"12\" instead of 12.")
        .add_description_line("Can be used for big numbers and acurate calculates.")
        .add_function(get_normelize())
        .add_function(get_add())
        .add_function(get_take_away())
        .add_function(get_times())
        .add_function(get_divide())
        .add_function(get_reminder())
        .add_function(get_eq())
        .add_function(get_neq())
        .add_function(get_gt())
        .add_function(get_lt())
        .add_function(get_gte())
        .add_function(get_lte())
        .add_function(get_abs())
        .add_function(get_round())
        .add_function(get_sort_by())
}
