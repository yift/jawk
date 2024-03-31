use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("?", 3, 3, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Boolean(true)) => self.0.apply(value, 1),
                    Some(JsonValue::Boolean(false)) => self.0.apply(value, 2),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("if")
    .add_description_line(
        "Return the second argument if the first argument is true. Return the third argument",
    )
    .add_description_line(
        "if the first is false. Return nothing if the first argument is not Boolean",
    )
    .add_example(
        Example::new()
            .add_argument("true")
            .add_argument("12")
            .add_argument("22")
            .expected_output("12"),
    )
    .add_example(
        Example::new()
            .add_argument("false")
            .add_argument("12")
            .add_argument("22")
            .expected_output("22"),
    )
    .add_example(
        Example::new()
            .add_argument("(array? .)")
            .add_argument("#1")
            .add_argument("#2")
            .expected_output("2")
            .input("[1, 2, 3]"),
    )
    .add_example(
        Example::new()
            .add_argument("(null? .)")
            .add_argument("#1")
            .add_argument("#2")
            .expected_output("3")
            .input("[1, 2, 3]"),
    )
    .add_example(
        Example::new()
            .add_argument("100")
            .add_argument("true")
            .add_argument("false"),
    )
}
