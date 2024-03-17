use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

use indexmap::IndexMap;

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("sub", 3, 3, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                let start = if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                    if let Ok(start) = TryInto::<usize>::try_into(n) {
                        start
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };
                let length = if let Some(JsonValue::Number(n)) = self.0.apply(value, 2) {
                    if let Ok(start) = TryInto::<usize>::try_into(n) {
                        start
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };
                match self.0.apply(value, 0) {
                    Some(JsonValue::Object(map)) => {
                        let mut new_map = IndexMap::with_capacity(length);
                        for (index, (k, v)) in map.into_iter().enumerate() {
                            if new_map.len() == length {
                                break;
                            }
                            if index >= start {
                                new_map.insert(k, v);
                            }
                        }
                        Some(new_map.into())
                    }
                    Some(JsonValue::Array(vec)) => {
                        let mut new_vec = Vec::with_capacity(length);
                        for (index, i) in vec.into_iter().enumerate() {
                            if new_vec.len() == length {
                                break;
                            }
                            if index >= start {
                                new_vec.push(i);
                            }
                        }
                        Some(new_vec.into())
                    }
                    Some(JsonValue::String(str)) => {
                        if start >= str.len() || length == 0 {
                            Some(String::new().into())
                        } else {
                            let last_index = start + length;
                            let last_index = if last_index >= str.len() {
                                str.len()
                            } else {
                                last_index
                            };
                            let str = str[start..last_index].to_string();
                            Some(str.into())
                        }
                    }
                    _ => None,
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_description_line(
            "If the first argument is a list, creates a new list that start from the second arguments and has the size of the third argument."
        )
        .add_description_line(
            "If the first argument is an object, creates a new object that start from the second arguments and has the size of the third argument."
        )
        .add_description_line(
            "If the first argument is a string, creates a substring that start from the second arguments and has the size of the third argument."
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, 4, 5, 6]")
                .add_argument("2")
                .add_argument("3")
                .expected_output("[3, 4, 5]")
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, 4]")
                .add_argument("6")
                .add_argument("10")
                .expected_output("[]")
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, 4]")
                .add_argument("0")
                .add_argument("10")
                .expected_output("[1, 2, 3, 4]")
        )
        .add_example(
            Example::new()
                .add_argument("[1, 2, 3, 4]")
                .add_argument("1")
                .add_argument("10")
                .expected_output("[2, 3, 4]")
        )
        .add_example(
            Example::new()
                .add_argument(
                    "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                )
                .add_argument("1")
                .add_argument("2")
                .expected_output("{\"key-2\": 2, \"key-3\": 3}")
        )
        .add_example(
            Example::new()
                .add_argument(
                    "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                )
                .add_argument("0")
                .add_argument("2")
                .expected_output("{\"key-1\": 1, \"key-2\": 2}")
        )
        .add_example(
            Example::new()
                .add_argument(
                    "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                )
                .add_argument("4")
                .add_argument("10")
                .expected_output("{\"key-5\": 5, \"key-6\": 6}")
        )
        .add_example(
            Example::new()
                .add_argument(
                    "{\"key-1\": 1, \"key-2\": 2, \"key-3\": 3, \"key-4\": 4, \"key-5\": 5, \"key-6\": 6}"
                )
                .add_argument("20")
                .add_argument("10")
                .expected_output("{}")
        )
        .add_example(
            Example::new()
                .add_argument("\"123456\"")
                .add_argument("1")
                .add_argument("3")
                .expected_output("\"234\"")
        )
        .add_example(
            Example::new()
                .add_argument("\"123456\"")
                .add_argument("1")
                .add_argument("30")
                .expected_output("\"23456\"")
        )
        .add_example(
            Example::new()
                .add_argument("\"123456\"")
                .add_argument("0")
                .add_argument("30")
                .expected_output("\"123456\"")
        )
        .add_example(
            Example::new()
                .add_argument("\"123456\"")
                .add_argument("20")
                .add_argument("30")
                .expected_output("\"\"")
        )
        .add_example(
            Example::new()
                .add_argument("\"123456\"")
                .add_argument("2")
                .add_argument("0")
                .expected_output("\"\"")
        )
        .add_example(Example::new().add_argument("50").add_argument("0").add_argument("10"))
        .add_example(
            Example::new().add_argument("\"123\"").add_argument("false").add_argument("10")
        )
        .add_example(
            Example::new().add_argument("\"123\"").add_argument("10").add_argument("{}")
        )
}
