use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("get", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(map)) => {
                        if let Some(JsonValue::String(key)) = self.0.apply(value, 1) {
                            map.get(&key).cloned()
                        } else {
                            None
                        }
                    }
                    Some(JsonValue::Array(array)) => {
                        if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                            if let Ok(index) = TryInto::<usize>::try_into(n) {
                                array.get(index).cloned()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("[]")
    .add_description_line("Get an item from an array by index or from a map by key.")
    .add_example(
        Example::new()
            .add_argument("[\"a\", \"b\", \"c\"]")
            .add_argument("1")
            .expected_output("\"b\""),
    )
    .add_example(
        Example::new()
            .add_argument("{\"key-1\": 12, \"key-2\": 32}")
            .add_argument("\"key-1\"")
            .expected_output("12"),
    )
    .add_example(
        Example::new()
            .add_argument("[\"a\", \"b\", \"c\"]")
            .add_argument("100"),
    )
}
