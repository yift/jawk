use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("number?", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Number(_)) => Some(true.into()),
                    _ => Some(false.into()),
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("return true if the argument is a number.")
    .add_example(
        Example::new()
            .add_argument("\"str\"")
            .expected_output("false"),
    )
    .add_example(Example::new().add_argument("1.32").expected_output("true"))
}
