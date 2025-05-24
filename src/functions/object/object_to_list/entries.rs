use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions},
    json_value::JsonValue,
    processor::Context,
    selection::Get,
};

pub fn get() -> FunctionDefinitions {
    FunctionDefinitions::new("entries", 1, 1, |args| {
        struct Impl(Vec<Rc<dyn Get>>);
        impl Get for Impl {
            fn get(&self, value: &Context) -> Option<JsonValue> {
                if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                    let mut list = Vec::with_capacity(map.len());
                    for (k, v) in map {
                        let mut data = IndexMap::with_capacity(2);
                        data.insert("value".to_string(), v.clone());
                        data.insert("key".to_string(), k.into());
                        list.push(data.into());
                    }
                    Some(JsonValue::Array(list))
                } else {
                    None
                }
            }
        }
        Rc::new(Impl(args))
    })
        .add_alias("to_list")
        .add_description_line(
            "Get the list of all the entries of an object. Each item of the list will be an object with `key` and `value` entries"
        )
        .add_example(
            Example::new()
                .add_argument(r#"{"key-1": 1, "key-2": false}"#)
                .expected_output(
                    r#"[{"key": "key-1", "value": 1}, {"key": "key-2", "value": false}]"#
                )
        )
        .add_example(Example::new().add_argument("[1, 2, 4]"))
}
