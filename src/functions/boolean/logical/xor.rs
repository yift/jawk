use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("xor", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(JsonValue::Boolean(val1)), Some(JsonValue::Boolean(val2))) =
                    (self.0.apply(value, 0), self.0.apply(value, 1))
                {
                    let eq = val1 ^ val2;
                    Some(eq.into())
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("^")
    .add_description_line("Return true if one, and only one, of the argument is true.")
    .add_example(
        Example::new()
            .add_argument("true")
            .add_argument("true")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("true")
            .add_argument("false")
            .expected_output("true"),
    )
    .add_example(
        Example::new()
            .add_argument("false")
            .add_argument("true")
            .expected_output("true"),
    )
    .add_example(
        Example::new()
            .add_argument("false")
            .add_argument("false")
            .expected_output("false"),
    )
    .add_example(Example::new().add_argument("null").add_argument("false"))
    .add_example(Example::new().add_argument("true").add_argument("12"))
}
