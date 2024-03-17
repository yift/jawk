use std::rc::Rc;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

use indexmap::IndexMap;

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("take", 2, 2, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Number(n)) = self.0.apply(value, 1) {
                    if let Ok(size) = TryInto::<usize>::try_into(n) {
                        match self.0.apply(value, 0) {
                            Some(JsonValue::Object(map)) => {
                                let map = if size > map.len() {
                                    map
                                } else {
                                    let mut new_map = IndexMap::with_capacity(size);
                                    for (k, v) in map {
                                        new_map.insert(k, v);
                                        if new_map.len() == size {
                                            break;
                                        }
                                    }
                                    new_map
                                };
                                Some(map.into())
                            }
                            Some(JsonValue::Array(vec)) => {
                                let vec = if size > vec.len() {
                                    vec
                                } else {
                                    let mut new_vec = Vec::with_capacity(size);
                                    for i in vec {
                                        new_vec.push(i);
                                        if new_vec.len() == size {
                                            break;
                                        }
                                    }
                                    new_vec
                                };
                                Some(vec.into())
                            }
                            Some(JsonValue::String(str)) => {
                                let str = if size > str.len() {
                                    str
                                } else {
                                    str[..size].into()
                                };
                                Some(str.into())
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
    .add_alias("take_first")
    .add_description_line("Take the first N of element in an array, object of string")
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .add_argument("2")
            .expected_output("[1, 2]"),
    )
    .add_example(
        Example::new()
            .add_argument("[1, 2, 3, 4]")
            .add_argument("6")
            .expected_output("[1, 2, 3, 4]"),
    )
    .add_example(
        Example::new()
            .add_argument("{\"key-1\": 1, \"key-2\": false}")
            .add_argument("1")
            .expected_output("{\"key-1\": 1}"),
    )
    .add_example(
        Example::new()
            .add_argument("{\"key-1\": 1, \"key-2\": false}")
            .add_argument("3")
            .expected_output("{\"key-1\": 1, \"key-2\": false}"),
    )
    .add_example(
        Example::new()
            .add_argument("\"123\"")
            .add_argument("2")
            .expected_output("\"12\""),
    )
    .add_example(
        Example::new()
            .add_argument("\"123\"")
            .add_argument("20")
            .expected_output("\"123\""),
    )
    .add_example(Example::new().add_argument("50").add_argument("10"))
    .add_example(Example::new().add_argument("\"123\"").add_argument("false"))
}
