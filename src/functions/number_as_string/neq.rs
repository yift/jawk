use std::rc::Rc;

use crate::{
    functions::number_as_string::to_big_decimal::BigDecimalConvert,
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("\"!=\"", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(val1), Some(val2)) = (
                    self.0.apply(value, 0).to_big_decimal(),
                    self.0.apply(value, 1).to_big_decimal(),
                ) {
                    let eq = !val1.eq(&val2);
                    Some(eq.into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("\"<>\"")
    .add_description_line("Compare two number as strings and return true if both are not equals.")
    .add_example(
        Example::new()
            .add_argument("\"1E400\"")
            .add_argument("\"3E400\"")
            .expected_output("true"),
    )
    .add_example(
        Example::new()
            .add_argument("\"1E99\"")
            .add_argument("\"1E99\"")
            .expected_output("false"),
    )
    .add_example(Example::new().add_argument("\"1\"").add_argument("1"))
    .add_example(Example::new().add_argument("1").add_argument("\"1\""))
}
