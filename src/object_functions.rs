use crate::{
    functions_definitions::{Arguments, Example, FunctionDefinitions, FunctionsGroup},
    json_value::JsonValue,
    selection::Get,
};

pub fn get_object_functions() -> FunctionsGroup {
    FunctionsGroup {
        name: "Object functions",
        functions: vec![
            FunctionDefinitions {
                name: "keys",
                aliases: vec![],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                                Some(JsonValue::Array(
                                    map.keys().cloned().map(JsonValue::String).collect(),
                                ))
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Get the list of keys from an object."],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}"],
                }],
            },
            FunctionDefinitions {
                name: "values",
                aliases: vec!["vals"],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            if let Some(JsonValue::Object(map)) = self.0.apply(value, 0) {
                                Some(JsonValue::Array(map.values().cloned().collect()))
                            } else {
                                None
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec!["Get the list of values from an object."],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["{\"key-1\": 1, \"key-2\": false}"],
                }],
            },
        ],
    }
}
