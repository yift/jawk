use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("object?", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(_)) => Some(true.into()),
                    _ => Some(false.into()),
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("map?")
    .add_alias("hash?")
    .add_description_line("return true if the argument is an object.")
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .expected_output("false"),
    )
    .add_example(
        Example::new()
            .add_argument("{\"key\": 12}")
            .expected_output("true"),
    )
}
