use std::rc::Rc;

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"abs\"", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                self.0
                    .apply(value, 0)
                    .to_big_decimal()
                    .map(|number| number.abs().into())
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("nas_abs")
    .add_description_line(
        "If the argument is numnber as string, return it's absolute value as string.",
    )
    .add_example(
        Example::new()
            .add_argument("\"100\"")
            .expected_output("\"100\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"-100\"")
            .expected_output("\"100\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"100e500\"")
            .expected_output("\"1e+502\""),
    )
    .add_example(Example::new().add_argument("0"))
    .add_example(Example::new().add_argument("\"test\""))
}
