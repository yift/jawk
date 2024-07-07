use std::rc::Rc;

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"round\"", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                self.0
                    .apply(value, 0)
                    .to_big_decimal()
                    .map(|number| number.round(0).into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nas_round")
    .add_description_line("If the argument is string as number, return it's rounded.")
    .add_example(
        Example::new()
            .add_argument("\"10.3\"")
            .expected_output("\"10\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"-10.3\"")
            .expected_output("\"-10\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"-10\"")
            .expected_output("\"-10\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"-10.5\"")
            .expected_output("\"-10\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"10.5\"")
            .expected_output("\"10\""),
    )
    .add_example(Example::new().add_argument("10"))
}
