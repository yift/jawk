mod nas_arithmetic;
mod nas_compare;
mod to_big_decimal;

use crate::functions_definitions::FunctionsGroup;
use nas_arithmetic::group as nas_arithmetic;
use nas_compare::group as nas_compare;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("nas")
        .add_description_line("NAS - Number As String, i.e. number that represent as string, like \"12\" instead of 12.")
        .add_description_line("Can be used for big numbers and accurate calculates.")
        .add_sub_group(nas_arithmetic())
        .add_sub_group(nas_compare())
}
