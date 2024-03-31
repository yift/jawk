use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("size", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(map)) => Some(map.len().into()),
                    Some(JsonValue::Array(list)) => Some(list.len().into()),
                    Some(JsonValue::String(str)) => Some(str.len().into()),
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("count")
    .add_alias("length")
    .add_alias("len")
    .add_description_line("Get the number of element in an array,")
    .add_description_line("the number of keys in an object or the number of characters")
    .add_description_line("in a string.")
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .expected_output("4"),
    )
    .add_example(
        Example::new()
            .add_argument("{\"key-1\": 1, \"key-2\": false}")
            .expected_output("2"),
    )
    .add_example(Example::new().add_argument("\"123\"").expected_output("3"))
    .add_example(
        Example::new()
            .add_argument("50")
            .explain("50 is not an array, not an object nor a string."),
    )
}
