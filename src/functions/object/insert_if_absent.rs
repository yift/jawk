use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("insert_if_absent", 3, 3, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let (Some(JsonValue::Object(map)), Some(JsonValue::String(key)), Some(val)) = (
                    self.0.apply(value, 0),
                    self.0.apply(value, 1),
                    self.0.apply(value, 2),
                ) {
                    if map.contains_key(&key) {
                        Some(map.into())
                    } else {
                        let mut new_map = map.clone();
                        new_map.insert(key, val);
                        Some(new_map.into())
                    }
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("put_if_absent")
    .add_alias("replace_if_absent")
    .add_alias("{-}")
    .add_description_line("Add a new entry to a map if it has no such key.")
    .add_description_line("The first argument should be an object.")
    .add_description_line("The second argument should be a key.")
    .add_description_line("The third argument should be a value.")
    .add_description_line("If the object has that key, it will not be replaced.")
    .add_example(
        Example::new()
            .add_argument("{}")
            .add_argument("\"a\"")
            .add_argument("1")
            .expected_output("{\"a\": 1}"),
    )
    .add_example(
        Example::new()
            .add_argument("{\"a\": 10, \"b\": 22}")
            .add_argument("\"a\"")
            .add_argument("-1")
            .expected_output("{\"a\": 10, \"b\": 22}")
            .explain("the object already has a value with the `a` key."),
    )
    .add_example(
        Example::new()
            .add_argument("[]")
            .add_argument("\"a\"")
            .add_argument("1"),
    )
    .add_example(
        Example::new()
            .add_argument("{}")
            .add_argument("1")
            .add_argument("1"),
    )
    .add_example(
        Example::new()
            .add_argument("{}")
            .add_argument("\"1\"")
            .add_argument("({} {} 1 1)"),
    )
}
