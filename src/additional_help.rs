use clap::builder::{PossibleValue, PossibleValuesParser};

use crate::{
    functions_definitions::{create_possible_fn_help_types, print_fn_help},
    selection_help::print_selection_help,
};

pub fn create_possible_values() -> PossibleValuesParser {
    let mut values = create_possible_fn_help_types();
    values.insert(
        0,
        PossibleValue::new("selection").help("Additional help about the selection"),
    );
    values.into()
}
pub fn display_additional_help(help_type: &str) {
    match help_type {
        "selection" => print_selection_help(),
        _ => print_fn_help(help_type),
    }
}
