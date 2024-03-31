use std::rc::Rc;

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\">=\"", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(val1), Some(val2)) = (self.0.apply(value, 0).to_big_decimal(), self.0.apply(value, 1).to_big_decimal()) {
                    let eq = val1 >= val2;
                    Some(eq.into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line(
        "Compare two numbers and string and return true if the first is greater or eqauls than the second.",
    )
    .add_example(
        Example::new()
            .add_argument("\"1\"")
            .add_argument("\"3\"")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("\"1\"")
            .add_argument("\"1\"")
            .expected_output("true"),
    )
    .add_example(
        Example::new()
            .add_argument("\"1E300\"")
            .add_argument("\"1E200\"")
            .expected_output("true"),
    )
    .add_example(
        Example::new()
            .add_argument("30")
            .add_argument("\"1E200\"")
    )
    .add_example(
        Example::new()
            .add_argument("\"1E200\"")
            .add_argument("30")
    )
}
