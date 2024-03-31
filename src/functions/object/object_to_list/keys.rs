use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("keys", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                    Some(JsonValue::Array(
                        map.keys().cloned().map(JsonValue::String).collect(),
                    ))
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_description_line("Get the list of keys from an object.")
    .add_example(
        Example::new()
            .add_argument("{\"key-1\": 1, \"key-2\": false}")
            .expected_output("[\"key-1\",\"key-2\"]"),
    )
    .add_example(Example::new().add_argument("[1, 2, 4]"))
}
