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
            FunctionDefinitions {
                name: "sort_by_keys",
                aliases: vec!["order_by_keys"],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            match self.0.apply(value, 0) {
                                Some(JsonValue::Object(map)) => {
                                    let mut map = map.clone();
                                    map.sort_keys();

                                    Some(map.into())
                                }
                                _ => None,
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Sort an object by it's keys.",
                    "If the first argument is an object, return object sorted by it's keys.",
                ],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["{\"z\": 1, \"x\": 2, \"w\": null}"],
                }],
            },
            FunctionDefinitions {
                name: "sort_by_values",
                aliases: vec!["order_by_values"],
                min_args_count: 1,
                max_args_count: 1,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            match self.0.apply(value, 0) {
                                Some(JsonValue::Object(map)) => {
                                    let mut map = map.clone();
                                    map.sort_by(|_, v1, _, v2| v1.cmp(v2));

                                    Some(map.into())
                                }
                                _ => None,
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Sort an object by it's values.",
                    "If the first argument is an object, return object sorted by it's values.",
                ],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["{\"z\": 5, \"x\": 2, \"w\": null}"],
                }],
            },
            FunctionDefinitions {
                name: "sort_by_values_by",
                aliases: vec!["order_by_values_by"],
                min_args_count: 2,
                max_args_count: 2,
                build_extractor: |args| {
                    struct Impl(Arguments);
                    impl Get for Impl {
                        fn get(&self, value: &Option<JsonValue>) -> Option<JsonValue> {
                            match self.0.apply(value, 0) {
                                Some(JsonValue::Object(map)) => {
                                    let mut map = map.clone();
                                    map.sort_by(|_, v1, _, v2| {
                                        let v1 = Some(v1.clone());
                                        let v1 = self.0.apply(&v1,1 );
                                        let v2 = Some(v2.clone());
                                        let v2 = self.0.apply(&v2,1 );
                                        v1.cmp(&v2)}
                                        );

                                    Some(map.into())
                                }
                                _ => None,
                            }
                        }
                    }
                    Box::new(Impl(Arguments::new(args)))
                },
                description: vec![
                    "Sort an object by a function to it's values.",
                    "If the first argument is an object, return object sorted by applying the second argumetn to it's values.",
                ],
                examples: vec![Example {
                    input: None,
                    arguments: vec!["{\"a\": [1, 2, 3], \"b\": [1], \"c\": [2], \"d\": [3], \"e\": [0, null, 0]}", "(.len)"],
                }],
            },
        ],
    }
}
